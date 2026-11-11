use std::sync::LazyLock;

use arcadia_common::error::{Error, Result};
use regex::Regex;
use url::{Host, Url};

use crate::services::ip_check::is_public;

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
pub fn matches_approved_host(host: &str, path: &str, approved: &str) -> bool {
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

/// Referrer policy of a CSS sheet, which applies to every url it loads at once: the origin is only
/// sent when all its absolute urls (`//host` ones included) are under `referrer_allowed_domains`.
pub fn css_referrer_policy(css: &str, referrer_allowed_domains: &[String]) -> &'static str {
    // every absolute url anywhere in the sheet, comments included, to never miss one. Browsers read
    // `\` as `/` and accept any number of slashes after the scheme, a malformed match only parses
    // to a host that is not allowed.
    static ABSOLUTE_URL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(?i)(?:https?:|[/\\]{2})[^\s'"()]*"#).unwrap());
    // CSS escapes (`\/`, `\2f`, `\5c`...) can spell slashes the regex above cannot see
    static ESCAPED_SLASH: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\\(?:[/\\]|0*(?:2f|5c))").unwrap());
    static BASE: LazyLock<Url> = LazyLock::new(|| Url::parse("https://relative.invalid").unwrap());
    let all_allowed = !referrer_allowed_domains.is_empty()
        && !ESCAPED_SLASH.is_match(css)
        && ABSOLUTE_URL.find_iter(css).all(|found| {
            BASE.join(found.as_str()).is_ok_and(|url| {
                url.host_str().is_some_and(|host| {
                    referrer_allowed_domains
                        .iter()
                        .any(|allowed| matches_approved_host(host, url.path(), allowed))
                })
            })
        });
    if all_allowed {
        "strict-origin"
    } else {
        "no-referrer"
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
    fn css_referrer_policy_needs_every_url_allowed() {
        let allowed = vec!["domain.com/elysia".to_string()];
        let only_allowed = ".a { background: url(https://domain.com/elysia/a.png) }
            .b { background: url('//domain.com/elysia/b.png') } .c { background: url(/local.png) }";
        assert_eq!(css_referrer_policy(only_allowed, &allowed), "strict-origin");
        for css in [
            ".a { background: url(https://domain.com/other/a.png) }",
            ".a { background: url(\"//evil.com/a.png\") }",
            "@import 'https://evil.com/a.css';",
            "/* https://evil.com */",
            ".a { background: url(https:/evil.com/a.png) }",
            ".a { background: url('https:\\\\evil.com/a.png') }",
            ".a { background: url(\\2f\\2f evil.com/a.png) }",
            ".a { background: url(\\/\\/evil.com/a.png) }",
        ] {
            assert_eq!(
                css_referrer_policy(&format!("{only_allowed} {css}"), &allowed),
                "no-referrer",
                "{css}"
            );
        }
        assert_eq!(css_referrer_policy(".a { color: red }", &[]), "no-referrer");
    }

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
