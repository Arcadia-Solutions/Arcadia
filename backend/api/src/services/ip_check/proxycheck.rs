use super::{non_empty, ProviderResult, REQUEST_TIMEOUT};
use arcadia_storage::models::ip_check::IpCheckResult;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Clone, Deserialize)]
pub struct ProxycheckConfig {
    pub priority: u32,
    pub api_key: Option<String>,
}

#[derive(Deserialize)]
struct Network {
    asn: Option<String>,
    provider: Option<String>,
}

#[derive(Deserialize)]
struct Location {
    country_name: Option<String>,
    country_code: Option<String>,
    region_name: Option<String>,
    city_name: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Deserialize)]
struct Detections {
    proxy: Option<bool>,
    vpn: Option<bool>,
    tor: Option<bool>,
    hosting: Option<bool>,
}

/// The entry of the checked ip address, the response being keyed by it.
#[derive(Deserialize)]
struct IpInformation {
    network: Network,
    location: Location,
    detections: Detections,
}

pub async fn check(
    client: &reqwest::Client,
    config: &ProxycheckConfig,
    ip: IpAddr,
) -> ProviderResult {
    let mut request = client
        .get(format!("https://proxycheck.io/v3/{ip}"))
        .timeout(REQUEST_TIMEOUT);
    if let Some(api_key) = &config.api_key {
        request = request.query(&[("key", api_key)]);
    }
    let response: serde_json::Value = request.send().await?.error_for_status()?.json().await?;
    into_result(response, ip)
}

fn into_result(mut response: serde_json::Value, ip: IpAddr) -> ProviderResult {
    // "warning" still comes with the information, "denied" and "error" do not
    let status = response["status"].as_str().unwrap_or_default();
    if status != "ok" && status != "warning" {
        return Err(format!("{status}: {}", response["message"]).into());
    }
    let information: IpInformation = serde_json::from_value(response[ip.to_string()].take())?;
    Ok(IpCheckResult {
        country_code: non_empty(information.location.country_code),
        country: non_empty(information.location.country_name),
        region: non_empty(information.location.region_name),
        city: non_empty(information.location.city_name),
        latitude: information.location.latitude,
        longitude: information.location.longitude,
        asn: non_empty(information.network.asn),
        organization: non_empty(information.network.provider),
        is_vpn: information.detections.vpn,
        is_proxy: information.detections.proxy,
        is_tor: information.detections.tor,
        is_hosting: information.detections.hosting,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_response() {
        let response = serde_json::from_str(
            r#"{"status":"ok","8.8.8.8":{"network":{"asn":"AS15169","provider":"Google LLC","type":"Business"},"location":{"country_name":"United States","country_code":"US","region_name":"California","city_name":"Mountain View","latitude":37.422,"longitude":-122.085},"detections":{"proxy":false,"vpn":true,"tor":false,"hosting":false}}}"#,
        )
        .unwrap();
        let result = into_result(response, "8.8.8.8".parse().unwrap()).unwrap();
        assert_eq!(result.city.as_deref(), Some("Mountain View"));
        assert_eq!(result.is_vpn, Some(true));
    }

    #[test]
    fn denied_is_an_error() {
        let response =
            serde_json::from_str(r#"{"status":"denied","message":"limit reached"}"#).unwrap();
        assert!(into_result(response, "8.8.8.8".parse().unwrap()).is_err());
    }
}
