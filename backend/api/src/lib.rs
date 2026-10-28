use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{arcadia_settings::ArcadiaSettings, notification::NotificationEvent},
    redis::RedisPoolInterface,
};
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

use crate::{config::Config, services::auth::Auth};

pub mod api_doc;
pub mod config;
pub mod handlers;
pub mod middlewares;
pub mod routes;
pub mod services;

pub struct Arcadia<R: RedisPoolInterface> {
    pub pool: Arc<ConnectionPool>,
    pub redis_pool: Arc<R>,
    pub auth: Auth<R>,
    pub settings: Arc<Mutex<ArcadiaSettings>>,
    pub notification_sender: broadcast::Sender<NotificationEvent>,
    /// Open notification streams per user id, to bound the sockets they hold.
    pub notification_streams: Arc<Mutex<HashMap<i32, usize>>>,
    /// HTTP client for external requests (scrapers, external APIs), optionally proxied.
    pub http_client: reqwest::Client,
    /// HTTP client for internal services (tracker, IRC, etc.), always bypasses proxy.
    pub internal_http_client: reqwest::Client,
    /// Every section is reachable directly on `Arcadia`, through its `Deref`.
    pub config: Config,
}

impl<R: RedisPoolInterface> Deref for Arcadia<R> {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

/// Builds a reqwest::Client that bypasses the proxy (for internal services like the tracker).
pub fn build_no_proxy_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("Failed to build no-proxy HTTP client")
}

/// Builds the HTTP client used for external requests (scrapers, external APIs).
/// When a proxy is configured, all requests (HTTP and HTTPS) are routed through it.
pub fn build_http_client(http_proxy: Option<&str>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder().no_proxy();

    if let Some(proxy_url) = http_proxy {
        let proxy =
            reqwest::Proxy::all(proxy_url).expect("HTTP_PROXY contains an invalid proxy URL");
        builder = builder.proxy(proxy);
    }

    builder.build().expect("Failed to build HTTP client")
}

impl<R: RedisPoolInterface> Arcadia<R> {
    pub fn new(
        pool: Arc<ConnectionPool>,
        redis_pool: Arc<R>,
        config: Config,
        settings: ArcadiaSettings,
    ) -> Self {
        let (notification_sender, _) = broadcast::channel(256);
        let http_client = build_http_client(config.api.http_proxy.as_deref());

        for plugin in &config.scrapers {
            log::info!("External source plugin registered: {}", plugin.id);
        }

        let internal_http_client = pool.internal_http_client.clone();

        Self {
            pool,
            auth: Auth::new(Arc::clone(&redis_pool)),
            redis_pool,
            settings: Arc::new(Mutex::new(settings)),
            notification_sender,
            notification_streams: Arc::new(Mutex::new(HashMap::new())),
            http_client,
            internal_http_client,
            config,
        }
    }
}
