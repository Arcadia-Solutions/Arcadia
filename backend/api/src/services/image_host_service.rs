use std::time::Duration;

use arcadia_common::error::{Error, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::{multipart, Client, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize};

use crate::{
    config::Config,
    services::{image_service::is_public_url, ip_check::is_public},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Where images are uploaded.
#[derive(Clone)]
pub enum ImageHost {
    /// Uploads are made on behalf of the user, authenticated with the OIDC client credentials.
    Elysia {
        url: String,
        client_id: String,
        client_secret: String,
    },
    /// Deprecated: every upload is made with the same API key, the uploader is not kept.
    Chevereto { api_url: String, api_key: String },
}

/// The configured image host: Elysia when `image_host.elysia_url` and `oidc` are both set,
/// otherwise Chevereto. Warns about a half configured Elysia and about Chevereto's deprecation.
pub fn configured_image_host(config: &Config) -> Option<ImageHost> {
    if let Some(url) = &config.image_host.elysia_url {
        match &config.oidc {
            Some(oidc) => {
                return Some(ImageHost::Elysia {
                    url: url.trim_end_matches('/').to_string(),
                    client_id: oidc.client_id.clone(),
                    client_secret: oidc.client_secret.clone(),
                })
            }
            None => log::warn!(
                "image_host.elysia_url is set but the oidc section is missing, Elysia uploads are disabled"
            ),
        }
    }
    let (Some(api_url), Some(api_key)) = (
        &config.image_host.chevereto_api_url,
        &config.image_host.chevereto_api_key,
    ) else {
        return None;
    };
    log::warn!("Chevereto image hosting is deprecated, switch to Elysia (image_host.elysia_url)");
    Some(ImageHost::Chevereto {
        api_url: api_url.clone(),
        api_key: api_key.clone(),
    })
}

/// The Arcadia user an image is uploaded for.
pub struct Uploader {
    pub id: i32,
    pub username: String,
}

/// What to upload: the image itself with its file name, or a link to an image to rehost.
enum ImageSource<'a> {
    Bytes { data: &'a [u8], file_name: &'a str },
    Url(&'a str),
}

/// Elysia's upload response, also the `image` object of Chevereto's.
#[derive(Deserialize)]
struct UploadedImage {
    url: String,
}

#[derive(Deserialize)]
struct CheveretoResponse {
    image: UploadedImage,
}

fn build_client() -> Result<Client> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .no_proxy()
        .build()
        .map_err(|e| Error::ImageHostUploadFailed(e.to_string()))
}

async fn send<T: DeserializeOwned>(request: RequestBuilder) -> Result<T> {
    let response = request
        .send()
        .await
        .map_err(|e| Error::ImageHostUploadFailed(e.to_string()))?;
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
    response
        .json()
        .await
        .map_err(|e| Error::ImageHostUploadFailed(e.to_string()))
}

async fn upload_with_client(
    client: &Client,
    image_host: &ImageHost,
    uploader: &Uploader,
    source: ImageSource<'_>,
) -> Result<String> {
    match image_host {
        ImageHost::Elysia {
            url,
            client_id,
            client_secret,
        } => {
            let request = match source {
                ImageSource::Bytes { data, file_name } => client
                    .post(format!("{url}/api/upload"))
                    .multipart(multipart::Form::new().part(
                        "file",
                        multipart::Part::bytes(data.to_vec()).file_name(file_name.to_string()),
                    )),
                ImageSource::Url(image_url) => client
                    .post(format!("{url}/api/upload-url"))
                    .json(&serde_json::json!({ "url": image_url })),
            };
            let request = request
                .basic_auth(client_id, Some(client_secret))
                .header("On-Behalf-Of-Sub", uploader.id.to_string())
                .header("On-Behalf-Of-Username", &uploader.username);
            Ok(send::<UploadedImage>(request).await?.url)
        }
        ImageHost::Chevereto { api_url, api_key } => {
            let source = match source {
                ImageSource::Bytes { data, .. } => BASE64.encode(data),
                ImageSource::Url(image_url) => image_url.to_string(),
            };
            let request = client.post(api_url).form(&[
                ("key", api_key.as_str()),
                ("source", &source),
                ("format", "json"),
            ]);
            Ok(send::<CheveretoResponse>(request).await?.image.url)
        }
    }
}

/// Uploads the image, returning its URL on the image host.
pub async fn upload_image(
    image_host: &ImageHost,
    uploader: &Uploader,
    image_bytes: &[u8],
    file_name: &str,
) -> Result<String> {
    upload_with_client(
        &build_client()?,
        image_host,
        uploader,
        ImageSource::Bytes {
            data: image_bytes,
            file_name,
        },
    )
    .await
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
pub async fn rehost_image_urls(image_host: &ImageHost, uploader: &Uploader, urls: &mut [String]) {
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
        match upload_with_client(&client, image_host, uploader, ImageSource::Url(url)).await {
            Ok(new_url) => *url = new_url,
            Err(e) => log::warn!("Failed to rehost image {url}: {e}"),
        }
    }
}
