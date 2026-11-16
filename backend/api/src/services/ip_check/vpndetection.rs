use super::{ProviderResult, REQUEST_TIMEOUT, non_empty};
use arcadia_storage::models::ip_check::IpCheckResult;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Clone, Deserialize)]
pub struct VpndetectionConfig {
    pub priority: u32,
}

#[derive(Default, Deserialize)]
struct Location {
    country: Option<String>,
    country_name: Option<String>,
}

#[derive(Deserialize)]
struct Asn {
    asn: Option<u32>,
    name: Option<String>,
    /// `isp`, `hosting`, `education`, `government` or `business`
    #[serde(rename = "type")]
    kind: Option<String>,
}

#[derive(Deserialize)]
struct Response {
    is_vpn: Option<bool>,
    is_tor: Option<bool>,
    #[serde(default)]
    location: Location,
    asn: Option<Asn>,
}

pub async fn check(client: &reqwest::Client, ip: IpAddr) -> ProviderResult {
    let response: Response = client
        .get(format!("https://api.vpndetection.io/{ip}"))
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(into_result(response))
}

fn into_result(response: Response) -> IpCheckResult {
    let asn = response.asn;
    IpCheckResult {
        country_code: non_empty(response.location.country),
        country: non_empty(response.location.country_name),
        region: None,
        city: None,
        latitude: None,
        longitude: None,
        asn: asn
            .as_ref()
            .and_then(|asn| asn.asn)
            .map(|number| format!("AS{number}")),
        organization: asn.as_ref().and_then(|asn| non_empty(asn.name.clone())),
        is_vpn: response.is_vpn,
        is_proxy: None,
        is_tor: response.is_tor,
        is_hosting: asn.and_then(|asn| asn.kind).map(|kind| kind == "hosting"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_response() {
        let response = serde_json::from_str(
            r#"{"ip":"185.65.135.230","is_bogon":false,"is_vpn":true,"is_relay":false,"is_tor":false,"relay":{},"tor":{},"location":{"continent":"EU","continent_name":"Europe","country":"SE","country_name":"Sweden","timezone":"Europe/Stockholm"},"asn":{"asn":39351,"name":"31173 Services AB","domain":"31173.se","type":"isp","prefix":"185.65.135.0/24"}}"#,
        )
        .unwrap();
        let result = into_result(response);
        assert_eq!(result.country_code.as_deref(), Some("SE"));
        assert_eq!(result.asn.as_deref(), Some("AS39351"));
        assert_eq!(result.organization.as_deref(), Some("31173 Services AB"));
        assert_eq!(result.is_vpn, Some(true));
        assert_eq!(result.is_hosting, Some(false));
        assert_eq!(result.is_proxy, None);
    }

    #[test]
    fn missing_network_is_unknown() {
        let response =
            serde_json::from_str(r#"{"ip":"2001:db8::1","is_vpn":false,"is_tor":false}"#).unwrap();
        let result = into_result(response);
        assert_eq!(result.asn, None);
        assert_eq!(result.is_hosting, None);
        assert_eq!(result.country_code, None);
    }
}
