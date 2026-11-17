use super::{ProviderResult, REQUEST_TIMEOUT, non_empty};
use arcadia_storage::models::ip_check::IpCheckResult;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Clone, Deserialize)]
pub struct VpndetectionConfig {
    pub priority: u32,
}

#[derive(Deserialize)]
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
    is_vpn: bool,
    is_relay: bool,
    is_tor: bool,
    location: Location,
    asn: Asn,
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
    let Asn { asn, name, kind } = response.asn;
    IpCheckResult {
        country_code: non_empty(response.location.country),
        country: non_empty(response.location.country_name),
        region: None,
        city: None,
        latitude: None,
        longitude: None,
        // unknown networks are sent as asn 0 with empty strings
        asn: asn
            .filter(|number| *number != 0)
            .map(|number| format!("AS{number}")),
        organization: non_empty(name),
        is_vpn: Some(response.is_vpn),
        is_proxy: Some(response.is_relay),
        is_tor: Some(response.is_tor),
        is_hosting: non_empty(kind).map(|kind| kind == "hosting"),
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
        assert_eq!(result.is_proxy, Some(false));
    }

    #[test]
    fn unknown_network_is_unknown() {
        let response = serde_json::from_str(
            r#"{"ip":"10.0.0.1","is_bogon":true,"is_vpn":false,"is_relay":false,"is_tor":false,"relay":{},"tor":{},"location":{"country":"","country_name":""},"asn":{"asn":0,"name":"","domain":"","type":"","prefix":""}}"#,
        )
        .unwrap();
        let result = into_result(response);
        assert_eq!(result.asn, None);
        assert_eq!(result.organization, None);
        assert_eq!(result.is_hosting, None);
        assert_eq!(result.country_code, None);
    }
}
