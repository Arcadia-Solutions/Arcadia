use arcadia_periodic_tasks::config::PeriodicTasksConfig;
use arcadia_shared::config::{
    default_log_level, deserialize_secret, DatabaseConfig, TelemetryConfig,
};
use reqwest::Url;
use serde::Deserialize;

use std::collections::BTreeMap;

use arcadia_storage::models::title_group::ContentType;

use crate::{handlers::scrapers::ExternalSource, services::ip_check::IpCheckProvidersConfig};

/// Sections of the `config.yml` file used by the API. The other ones are ignored.
#[derive(Clone, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub tracker: TrackerConfig,
    pub periodic_tasks: PeriodicTasksConfig,
    #[serde(default)]
    pub smtp: SmtpConfig,
    #[serde(default)]
    pub image_host: ImageHostConfig,
    #[serde(default)]
    pub ergo: ErgoConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    /// External sources provided by plugins, declared by the instance administrator.
    #[serde(default)]
    pub scrapers: Vec<ExternalSourcePlugin>,
    /// The service (Elysia) allowed to SSO its users via Arcadia's OpenID Connect provider.
    #[serde(default)]
    pub oidc: Option<OidcConfig>,
}

#[derive(Clone, Deserialize)]
pub struct OidcConfig {
    /// Public URL of `/api/oauth`, exactly as the client is configured with.
    pub issuer: Url,
    pub client_id: String,
    pub client_secret: String,
    /// The only redirect URI the client may use, matched exactly.
    #[serde(deserialize_with = "deserialize_redirect_uri")]
    pub redirect_uri: String,
}

/// Kept as the raw string for exact matching, but must be an absolute URL without a fragment,
/// since the authorization code is appended as query parameters (RFC 6749 §3.1.2).
fn deserialize_redirect_uri<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let redirect_uri = String::deserialize(deserializer)?;
    let url = Url::parse(&redirect_uri).map_err(serde::de::Error::custom)?;
    if url.fragment().is_some() {
        return Err(serde::de::Error::custom(
            "oidc.redirect_uri must not contain a fragment",
        ));
    }
    Ok(redirect_uri)
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_api_port() -> u16 {
    8080
}

#[derive(Clone, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_api_port")]
    pub port: u16,
    #[serde(deserialize_with = "deserialize_secret")]
    pub jwt_secret: String,
    pub frontend_url: Url,
    /// Overridden by the `RUST_LOG` environment variable when it is set.
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub tmdb_api_key: Option<String>,
    #[serde(default)]
    pub comic_vine_api_key: Option<String>,
    /// When set, outgoing requests to external services are routed through this proxy.
    /// Internal requests (tracker, irc, image host, etc.) are not affected.
    #[serde(default)]
    pub http_proxy: Option<String>,
    /// Biggest `page_size` accepted on the paginated endpoints. Bigger values are capped to it.
    pub max_page_size: u32,
    #[serde(default)]
    pub ip_check_providers: IpCheckProvidersConfig,
}

fn default_redis_port() -> u16 {
    6379
}

#[derive(Clone, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_host")]
    pub host: String,
    pub password: String,
    #[serde(default = "default_redis_port")]
    pub port: u16,
}

#[derive(Clone, Deserialize)]
pub struct TrackerConfig {
    pub name: String,
    pub url: Url,
    pub url_internal: Url,
    #[serde(deserialize_with = "deserialize_secret")]
    pub api_key: String,
    #[serde(default)]
    pub torrent_source_tag: Option<String>,
}

#[derive(Clone, Default, Deserialize)]
pub struct SmtpConfig {
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub from_email: Option<String>,
    #[serde(default)]
    pub from_name: Option<String>,
}

impl SmtpConfig {
    /// Emails are only sent when every key of the section is set.
    pub fn is_enabled(&self) -> bool {
        self.host.is_some()
            && self.port.is_some()
            && self.username.is_some()
            && self.password.is_some()
            && self.from_email.is_some()
            && self.from_name.is_some()
    }
}

#[derive(Clone, Default, Deserialize)]
pub struct ErgoConfig {
    #[serde(default)]
    pub api_url: Option<String>,
    #[serde(default)]
    pub api_bearer_token: Option<String>,
    /// Token that Ergo sends when calling the auth callback endpoint.
    #[serde(default)]
    pub auth_callback_token: Option<String>,
}

impl ErgoConfig {
    pub fn is_enabled(&self) -> bool {
        self.api_url.is_some() && self.api_bearer_token.is_some()
    }
}

#[derive(Clone, Default, Deserialize)]
pub struct ImageHostConfig {
    #[serde(default)]
    pub chevereto_api_url: Option<String>,
    #[serde(default)]
    pub chevereto_api_key: Option<String>,
    /// Automatically rehost images coming from external database scrapers.
    #[serde(default)]
    pub rehost_external_images: bool,
}

fn default_timeout_seconds() -> u64 {
    30
}

/// An external source (scraper) provided by a plugin running as a separate service.
/// Declared by the instance administrator in the `scrapers` section of the configuration file.
#[derive(Debug, Clone, Deserialize)]
pub struct ExternalSourcePlugin {
    /// Used as the last segment of `/api/external-sources/{source_id}`.
    pub id: String,
    pub placeholder: String,
    /// The websites the endpoint accepts links from, each with the content types it supports.
    /// A single endpoint may serve several websites, dispatching on the link it is given.
    pub sources: BTreeMap<String, Vec<ContentType>>,
    /// Endpoint of the plugin, called with the `url` query parameter.
    pub url: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

impl ExternalSourcePlugin {
    /// What the interface needs to display the source.
    pub fn external_source(&self) -> ExternalSource {
        ExternalSource {
            id: self.id.clone(),
            placeholder: self.placeholder.clone(),
            sources: self.sources.clone(),
        }
    }
}
