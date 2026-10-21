use super::{non_empty, ProviderResult, REQUEST_TIMEOUT};
use arcadia_storage::models::ip_check::IpCheckResult;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Clone, Deserialize)]
pub struct VpnapiConfig {
    pub priority: u32,
    pub api_key: String,
}

#[derive(Deserialize)]
struct Security {
    vpn: Option<bool>,
    proxy: Option<bool>,
    tor: Option<bool>,
}

#[derive(Deserialize)]
struct Location {
    country: Option<String>,
    country_code: Option<String>,
    region: Option<String>,
    city: Option<String>,
    /// sent as a string, e.g. `"37.7510"`
    latitude: Option<String>,
    longitude: Option<String>,
}

#[derive(Deserialize)]
struct Network {
    autonomous_system_number: Option<String>,
    autonomous_system_organization: Option<String>,
}

#[derive(Deserialize)]
struct Response {
    security: Security,
    location: Location,
    network: Network,
}

pub async fn check(client: &reqwest::Client, config: &VpnapiConfig, ip: IpAddr) -> ProviderResult {
    let response: Response = client
        .get(format!("https://vpnapi.io/api/{ip}"))
        .query(&[("key", &config.api_key)])
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(into_result(response))
}

fn into_result(response: Response) -> IpCheckResult {
    IpCheckResult {
        country_code: non_empty(response.location.country_code),
        country: non_empty(response.location.country),
        region: non_empty(response.location.region),
        city: non_empty(response.location.city),
        latitude: response
            .location
            .latitude
            .and_then(|latitude| latitude.parse().ok()),
        longitude: response
            .location
            .longitude
            .and_then(|longitude| longitude.parse().ok()),
        asn: non_empty(response.network.autonomous_system_number),
        organization: non_empty(response.network.autonomous_system_organization),
        is_vpn: response.security.vpn,
        is_proxy: response.security.proxy,
        is_tor: response.security.tor,
        is_hosting: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_response() {
        let response = serde_json::from_str(
            r#"{"ip":"8.8.8.8","security":{"vpn":false,"proxy":false,"tor":false,"relay":false},"location":{"city":"","region":"","country":"United States","continent":"North America","region_code":"","country_code":"US","continent_code":"NA","latitude":"37.7510","longitude":"-97.8220","time_zone":"America/Chicago","is_in_european_union":false},"network":{"network":"8.8.8.0/24","autonomous_system_number":"AS15169","autonomous_system_organization":"GOOGLE"}}"#,
        )
        .unwrap();
        let result = into_result(response);
        assert_eq!(result.city, None);
        assert_eq!(result.latitude, Some(37.751));
        assert_eq!(result.asn.as_deref(), Some("AS15169"));
    }
}
