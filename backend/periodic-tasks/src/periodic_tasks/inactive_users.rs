use arcadia_storage::{
    connection_pool::ConnectionPool, redis::RedisPoolInterface,
    services::token_invalidation_service::invalidate_user_tokens,
};
use std::sync::Arc;

pub async fn ban_inactive_users<R: RedisPoolInterface>(
    pool: Arc<ConnectionPool>,
    redis_pool: Arc<R>,
) -> Result<u64, Box<dyn std::error::Error>> {
    let settings = pool.get_arcadia_settings().await?;

    let Some(inactive_days) = settings.inactive_user_ban_after_days else {
        log::debug!("Inactive user ban is disabled, skipping");
        return Ok(0);
    };

    let banned_user_ids = pool.ban_inactive_users(inactive_days).await?;
    // the banned users must not keep using the tokens they were issued
    for user_id in &banned_user_ids {
        invalidate_user_tokens(&*redis_pool, *user_id).await?;
    }
    log::info!("Banned {} inactive users", banned_user_ids.len());
    Ok(banned_user_ids.len() as u64)
}
