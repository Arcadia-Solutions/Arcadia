use super::{non_empty, ProviderResult, REQUEST_TIMEOUT};
use arcadia_storage::models::ip_check::IpCheckResult;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Clone, Deserialize)]
pub struct IpApiConfig {
    pub priority: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    status: String,
    message: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    region_name: Option<String>,
    city: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    isp: Option<String>,
    /// e.g. `AS15169 Google LLC`
    #[serde(rename = "as")]
    autonomous_system: Option<String>,
    proxy: Option<bool>,
    hosting: Option<bool>,
}

pub async fn check(client: &reqwest::Client, ip: IpAddr) -> ProviderResult {
    let response: Response = client
        .get(format!(
            "http://ip-api.com/json/{ip}?fields=status,message,country,countryCode,regionName,city,lat,lon,isp,as,proxy,hosting"
        ))
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    into_result(response)
}

fn into_result(response: Response) -> ProviderResult {
    if response.status != "success" {
        return Err(response.message.unwrap_or(response.status).into());
    }
    Ok(IpCheckResult {
        country_code: non_empty(response.country_code),
        country: non_empty(response.country),
        region: non_empty(response.region_name),
        city: non_empty(response.city),
        latitude: response.lat,
        longitude: response.lon,
        asn: response
            .autonomous_system
            .as_deref()
            .and_then(|autonomous_system| autonomous_system.split_whitespace().next())
            .map(String::from),
        organization: non_empty(response.isp),
        is_vpn: None,
        is_proxy: response.proxy,
        is_tor: None,
        is_hosting: response.hosting,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_response() {
        let response = serde_json::from_str(
            r#"{"status":"success","country":"United States","countryCode":"US","regionName":"Virginia","city":"Ashburn","lat":39.03,"lon":-77.5,"isp":"Google LLC","as":"AS15169 Google LLC","proxy":true,"hosting":true}"#,
        )
        .unwrap();
        let result = into_result(response).unwrap();
        assert_eq!(result.country_code.as_deref(), Some("US"));
        assert_eq!(result.asn.as_deref(), Some("AS15169"));
        assert_eq!(result.is_hosting, Some(true));
        assert_eq!(result.is_vpn, None);
    }

    #[test]
    fn failure_is_an_error() {
        let response =
            serde_json::from_str(r#"{"status":"fail","message":"reserved range"}"#).unwrap();
        assert!(into_result(response).is_err());
    }
}
