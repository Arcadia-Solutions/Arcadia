use arcadia_common::error::{Error, Result};
use url::Url;

pub fn validate_image_urls(image_urls: &[String], approved_image_hosts: &[String]) -> Result<()> {
    if approved_image_hosts.is_empty() {
        return Ok(());
    }

    for image_url in image_urls {
        validate_image_url(image_url, approved_image_hosts)?;
    }
    Ok(())
}

pub fn validate_image_url(image_url: &str, approved_image_hosts: &[String]) -> Result<()> {
    if approved_image_hosts.is_empty() {
        return Ok(());
    }

    let parsed_url = Url::parse(image_url).map_err(|_| Error::ImageHostNotApproved {
        url: image_url.to_string(),
        approved_hosts: approved_image_hosts.to_vec(),
    })?;

    let image_host = parsed_url
        .host_str()
        .ok_or_else(|| Error::ImageHostNotApproved {
            url: image_url.to_string(),
            approved_hosts: approved_image_hosts.to_vec(),
        })?;

    if approved_image_hosts.iter().any(|h| h == image_host) {
        return Ok(());
    }

    Err(Error::ImageHostNotApproved {
        url: image_url.to_string(),
        approved_hosts: approved_image_hosts.to_vec(),
    })
}

// trailers end up in a <video>/<iframe> src, any other scheme (javascript:, data:...) must never be stored
pub fn validate_trailer_urls(trailer_urls: &[String]) -> Result<()> {
    for trailer_url in trailer_urls {
        if !Url::parse(trailer_url).is_ok_and(|url| matches!(url.scheme(), "http" | "https")) {
            return Err(Error::InvalidTrailerUrl(trailer_url.clone()));
        }
    }
    Ok(())
}
