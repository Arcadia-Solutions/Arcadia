use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::borrow::Borrow;
use url::Url;

#[derive(Clone)]
pub struct TrackerConfig {
    pub url_internal: Url,
    pub api_key: String,
}

pub struct ConnectionPool {
    pub(crate) pool: PgPool,
    pub tracker_config: TrackerConfig,
    pub internal_http_client: reqwest::Client,
}

impl ConnectionPool {
    pub async fn try_new(
        db_uri: &str,
        tracker_config: TrackerConfig,
        internal_http_client: reqwest::Client,
    ) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_uri)
            .await
            .expect("Error building a connection pool");

        Ok(Self {
            pool,
            tracker_config,
            internal_http_client,
        })
    }

    /// Creates a ConnectionPool from an existing PgPool, without tracker config.
    /// Used in tests where tracker notifications are disabled.
    pub fn with_pg_pool(pool: PgPool) -> Self {
        Self {
            pool,
            // Dummy config for tests - notifications are disabled via notify_tracker parameter
            tracker_config: TrackerConfig {
                url_internal: Url::parse("http://localhost").unwrap(),
                api_key: String::new(),
            },
            internal_http_client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("Failed to build no-proxy HTTP client"),
        }
    }

    /// Sends a request to the tracker's internal api (`/api/<path_segments>`),
    /// only logging a warning on failure so the caller's operation still succeeds.
    pub async fn notify_tracker(
        &self,
        method: reqwest::Method,
        path_segments: &[&str],
        body: Option<&impl Serialize>,
        action_description: &str,
    ) {
        let mut url = self.tracker_config.url_internal.clone();
        url.path_segments_mut()
            .unwrap()
            .push("api")
            .extend(path_segments);

        let mut request = self
            .internal_http_client
            .request(method, url)
            .header("x-api-key", self.tracker_config.api_key.clone());
        if let Some(body) = body {
            request = request.json(body);
        }

        if let Err(error) = request.send().await {
            log::warn!("Failed to {action_description} in tracker: {error}");
        }
    }
}

impl Borrow<PgPool> for ConnectionPool {
    fn borrow(&self) -> &PgPool {
        &self.pool
    }
}
