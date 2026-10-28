use crate::redis::{RedisInterface, RedisPoolInterface};
use arcadia_common::error::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::sync::LazyLock;

pub static REFRESH_TOKEN_DURATION: LazyLock<Duration> = LazyLock::new(|| Duration::days(90));

#[derive(Serialize, Deserialize)]
pub struct InvalidationEntry {
    pub user_id: i32,
    pub token_invalidation_ts: i64,
}

impl InvalidationEntry {
    pub fn new(user_id: i32) -> Self {
        let now = Utc::now();

        Self {
            user_id,
            token_invalidation_ts: now.timestamp(),
        }
    }
}

/// Invalidates every token issued to the user so far.
pub async fn invalidate_user_tokens<R: RedisPoolInterface>(
    redis_pool: &R,
    user_id: i32,
) -> Result<()> {
    let entry = InvalidationEntry::new(user_id);
    let mut redis = redis_pool.connection().await?;

    // add entry to the redis with a TTL of the refresh token so we know
    // for sure that it will be present for as long as the refresh token is present
    redis
        .set_ex(
            user_id,
            to_string(&entry)?,
            (*REFRESH_TOKEN_DURATION).as_seconds_f64() as usize,
        )
        .await?;
    Ok(())
}
