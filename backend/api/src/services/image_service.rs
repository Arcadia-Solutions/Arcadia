use arcadia_common::error::{Error, Result};
use url::{Host, Url};

use crate::services::ip_check::is_public;

pub fn validate_image_urls(image_urls: &[String], approved_image_hosts: &[String]) -> Result<()> {
    for image_url in image_urls {
        validate_image_url(image_url, approved_image_hosts)?;
    }
    Ok(())
}

pub fn validate_image_url(image_url: &str, approved_image_hosts: &[String]) -> Result<()> {
    if approved_image_hosts.is_empty() {
        // the image host fetches the urls it is asked to rehost, they must not reach internal services
        return match Url::parse(image_url) {
            Ok(url) if url.host().is_some() && !is_public_url(&url) => {
                Err(Error::NonPublicImageUrl(image_url.to_string()))
            }
            _ => Ok(()),
        };
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

    if approved_image_hosts
        .iter()
        .any(|approved| matches_approved_host(image_host, parsed_url.path(), approved))
    {
        return Ok(());
    }

    Err(Error::ImageHostNotApproved {
        url: image_url.to_string(),
        approved_hosts: approved_image_hosts.to_vec(),
    })
}

/// An approved entry is either a bare host (`domain.com`) or a host followed by a path prefix
/// (`domain.com/elysia`), in which case the url path must be that prefix or lie under it.
fn matches_approved_host(host: &str, path: &str, approved: &str) -> bool {
    let (approved_host, approved_path) = match approved.trim_end_matches('/').split_once('/') {
        Some((h, p)) => (h, Some(p)),
        None => (approved.trim_end_matches('/'), None),
    };
    if host != approved_host {
        return false;
    }
    match approved_path {
        None => true,
        Some(prefix) => path
            .strip_prefix('/')
            .and_then(|p| p.strip_prefix(prefix))
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/')),
    }
}

/// Whether the url can be fetched without reaching the internal network, judging from the url
/// alone: a public name can still resolve to an internal address.
pub fn is_public_url(url: &Url) -> bool {
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    match url.host() {
        Some(Host::Ipv4(ip)) => is_public(ip.into()),
        Some(Host::Ipv6(ip)) => is_public(ip.into()),
        Some(Host::Domain(domain)) => {
            let domain = domain.trim_end_matches('.').to_ascii_lowercase();
            // single label names (docker services, intranet hosts) and the internal suffixes
            domain.contains('.')
                && ![".local", ".localhost", ".internal", ".lan", ".home.arpa"]
                    .iter()
                    .any(|suffix| domain.ends_with(suffix))
        }
        None => false,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_hosts_with_path_prefix() {
        let approved = vec!["imgur.com".to_string(), "domain.com/elysia".to_string()];
        for url in [
            "https://imgur.com/a.png",
            "https://domain.com/elysia/a.png",
            "https://domain.com/elysia/sub/a.png",
        ] {
            assert!(validate_image_url(url, &approved).is_ok(), "{url}");
        }
        for url in [
            "https://domain.com/a.png",
            "https://domain.com/elysiafake/a.png",
            "https://domain.com/elysia/../other/a.png",
            "https://evil.com/elysia/a.png",
        ] {
            assert!(validate_image_url(url, &approved).is_err(), "{url}");
        }
    }

    #[test]
    fn only_public_urls_are_public() {
        for url in [
            "http://127.0.0.1:9099/canary",
            "http://169.254.169.254/latest/meta-data/",
            "http://[::1]/",
            "http://[::ffff:10.0.0.1]/",
            "http://backend:8080/api",
            "http://redis.local/",
            "http://metadata.google.internal/",
            "http://foo.localhost/",
            "file:///etc/passwd",
            "gopher://example.com/",
        ] {
            assert!(!is_public_url(&Url::parse(url).unwrap()), "{url}");
        }
        for url in ["https://example.com/a.png", "http://93.184.216.34/a.png"] {
            assert!(is_public_url(&Url::parse(url).unwrap()), "{url}");
        }
    }
}
