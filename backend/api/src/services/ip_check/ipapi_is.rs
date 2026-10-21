use super::{non_empty, ProviderResult, REQUEST_TIMEOUT};
use arcadia_storage::models::ip_check::IpCheckResult;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Clone, Deserialize)]
pub struct IpapiIsConfig {
    pub priority: u32,
    pub api_key: String,
}

#[derive(Default, Deserialize)]
struct AutonomousSystem {
    asn: Option<u32>,
    org: Option<String>,
}

#[derive(Default, Deserialize)]
struct Location {
    country: Option<String>,
    country_code: Option<String>,
    state: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Deserialize)]
struct Response {
    /// Only sent with a successful response, errors (invalid key, exhausted quota, etc.) are
    /// sent as `{"error": "..."}`, possibly with a 200 status code.
    ip: Option<String>,
    error: Option<String>,
    is_vpn: Option<bool>,
    is_proxy: Option<bool>,
    is_tor: Option<bool>,
    is_datacenter: Option<bool>,
    asn: Option<AutonomousSystem>,
    location: Option<Location>,
}

pub async fn check(client: &reqwest::Client, config: &IpapiIsConfig, ip: IpAddr) -> ProviderResult {
    let response: Response = client
        .get("https://api.ipapi.is/")
        .query(&[("q", ip.to_string()), ("key", config.api_key.clone())])
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    into_result(response)
}

fn into_result(response: Response) -> ProviderResult {
    if response.ip.is_none() {
        return Err(response
            .error
            .unwrap_or_else(|| "no ip in the response".to_string())
            .into());
    }
    let location = response.location.unwrap_or_default();
    let autonomous_system = response.asn.unwrap_or_default();
    Ok(IpCheckResult {
        country_code: non_empty(location.country_code),
        country: non_empty(location.country),
        region: non_empty(location.state),
        city: non_empty(location.city),
        latitude: location.latitude,
        longitude: location.longitude,
        asn: autonomous_system.asn.map(|asn| format!("AS{asn}")),
        organization: non_empty(autonomous_system.org),
        is_vpn: response.is_vpn,
        is_proxy: response.is_proxy,
        is_tor: response.is_tor,
        is_hosting: response.is_datacenter,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_response() {
        let response = serde_json::from_str(
            r#"{"ip":"8.8.8.8","is_vpn":false,"is_proxy":false,"is_tor":false,"is_datacenter":true,"asn":{"asn":15169,"org":"Google LLC"},"location":{"country":"United States","country_code":"US","state":"California","city":"Mountain View","latitude":37.38605,"longitude":-122.08385}}"#,
        )
        .unwrap();
        let result = into_result(response).unwrap();
        assert_eq!(result.asn.as_deref(), Some("AS15169"));
        assert_eq!(result.region.as_deref(), Some("California"));
        assert_eq!(result.is_hosting, Some(true));
    }

    #[test]
    fn error_is_an_error() {
        let response = serde_json::from_str(r#"{"error":"Invalid API key"}"#).unwrap();
        assert!(into_result(response).is_err());
    }
}
