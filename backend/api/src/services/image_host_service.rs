use std::time::Duration;

use arcadia_common::error::{Error, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::Client;
use serde::Deserialize;

use crate::{
    config::ImageHostConfig,
    services::{image_service::is_public_url, ip_check::is_public},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
struct CheveretoResponse {
    image: CheveretoImage,
}

#[derive(Deserialize)]
struct CheveretoImage {
    url: String,
}

fn build_client() -> Result<Client> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .no_proxy()
        .build()
        .map_err(|e| Error::ImageHostUploadFailed(e.to_string()))
}

async fn parse_chevereto_response(response: reqwest::Response) -> Result<String> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(Error::ImageHostUploadFailed(format!(
            "status {status}: {body}"
        )));
    }

    let chevereto_response: CheveretoResponse = response
        .json()
        .await
        .map_err(|e| Error::ImageHostUploadFailed(e.to_string()))?;

    Ok(chevereto_response.image.url)
}

pub async fn upload_image_to_chevereto(
    api_url: &str,
    api_key: &str,
    image_bytes: &[u8],
) -> Result<String> {
    let client = build_client()?;
    let encoded = BASE64.encode(image_bytes);

    let response = client
        .post(api_url)
        .form(&[("key", api_key), ("source", &encoded), ("format", "json")])
        .send()
        .await
        .map_err(|e| Error::ImageHostUploadFailed(e.to_string()))?;

    parse_chevereto_response(response).await
}

/// The image host fetches the url itself, from inside the internal network, so it must only be
/// given urls of public addresses.
async fn can_be_rehosted(url: &str) -> bool {
    let Ok(url) = url::Url::parse(url) else {
        return false;
    };
    if !is_public_url(&url) {
        return false;
    }
    let (Some(host), Some(port)) = (url.host_str(), url.port_or_known_default()) else {
        return false;
    };
    // ponytail: the image host resolves the name again, a rebinding dns can still answer differently
    match tokio::net::lookup_host((host.trim_matches(['[', ']']), port)).await {
        Ok(mut addresses) => addresses.all(|address| is_public(address.ip())),
        Err(_) => false,
    }
}

/// Rehosts all non-empty URLs in the slice, one by one.
/// Failures are logged and the original URL is kept.
pub async fn rehost_image_urls(config: &ImageHostConfig, urls: &mut [String]) {
    if !config.rehost_external_images {
        return;
    }

    let (Some(api_url), Some(api_key)) = (&config.chevereto_api_url, &config.chevereto_api_key)
    else {
        return;
    };

    let Ok(client) = build_client() else {
        return;
    };

    for url in urls.iter_mut() {
        if url.is_empty() {
            continue;
        }
        if !can_be_rehosted(url).await {
            log::warn!("Not rehosting image {url}: it does not point to a public address");
            continue;
        }
        let result = client
            .post(api_url.as_str())
            .form(&[
                ("key", api_key.as_str()),
                ("source", url.as_str()),
                ("format", "json"),
            ])
            .send()
            .await
            .map_err(|e| Error::ImageHostUploadFailed(e.to_string()));

        match result {
            Ok(response) => match parse_chevereto_response(response).await {
                Ok(new_url) => *url = new_url,
                Err(e) => log::warn!("Failed to rehost image {url}: {e}"),
            },
            Err(e) => log::warn!("Failed to rehost image {url}: {e}"),
        }
    }
}
