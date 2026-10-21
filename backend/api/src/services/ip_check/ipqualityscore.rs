//! IPQualityScore does not return the name of the country.

use super::{non_empty, ProviderResult, REQUEST_TIMEOUT};
use arcadia_storage::models::ip_check::IpCheckResult;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Clone, Deserialize)]
pub struct IpqualityscoreConfig {
    pub priority: u32,
    pub api_key: String,
}

#[derive(Deserialize)]
struct Response {
    success: bool,
    message: Option<String>,
    country_code: Option<String>,
    region: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    #[serde(rename = "ASN")]
    asn: Option<u32>,
    #[serde(rename = "ISP")]
    isp: Option<String>,
    proxy: Option<bool>,
    vpn: Option<bool>,
    tor: Option<bool>,
    /// Residential, Mobile, Corporate, Education or Data Center
    connection_type: Option<String>,
}

pub async fn check(
    client: &reqwest::Client,
    config: &IpqualityscoreConfig,
    ip: IpAddr,
) -> ProviderResult {
    let mut url = reqwest::Url::parse("https://ipqualityscore.com/api/json/ip")?;
    url.path_segments_mut()
        .map_err(|_| "invalid ipqualityscore url")?
        .push(&config.api_key)
        .push(&ip.to_string());
    let response: Response = client
        .get(url)
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    into_result(response)
}

fn into_result(response: Response) -> ProviderResult {
    // errors, including exhausted credits, are sent with a 200 status code
    if !response.success {
        return Err(response.message.unwrap_or_default().into());
    }
    Ok(IpCheckResult {
        country_code: non_empty(response.country_code),
        country: None,
        region: non_empty(response.region),
        city: non_empty(response.city),
        latitude: response.latitude,
        longitude: response.longitude,
        asn: response.asn.map(|asn| format!("AS{asn}")),
        organization: non_empty(response.isp),
        is_vpn: response.vpn,
        is_proxy: response.proxy,
        is_tor: response.tor,
        is_hosting: response
            .connection_type
            .map(|connection_type| connection_type == "Data Center"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_response() {
        let response = serde_json::from_str(
            r#"{"success":true,"message":"Success","fraud_score":0,"country_code":"US","region":"California","city":"Mountain View","ISP":"Google","ASN":15169,"latitude":37.39,"longitude":-122.07,"proxy":true,"vpn":false,"tor":false,"connection_type":"Data Center"}"#,
        )
        .unwrap();
        let result = into_result(response).unwrap();
        assert_eq!(result.asn.as_deref(), Some("AS15169"));
        assert_eq!(result.is_hosting, Some(true));
    }

    #[test]
    fn exhausted_credits_are_an_error() {
        let response = serde_json::from_str(
            r#"{"success":false,"message":"You have insufficient credits to make this query."}"#,
        )
        .unwrap();
        assert!(into_result(response).is_err());
    }
}
