pub mod common;
pub mod mocks;

use arcadia_api::services::auth::Auth;
use arcadia_periodic_tasks::periodic_tasks::inactive_users;
use arcadia_storage::connection_pool::ConnectionPool;
use chrono::Utc;
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_inactive_users"), migrations = "../storage/migrations")]
async fn test_ban_inactive_users_bans_only_eligible(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let banned_user_ids = pool.ban_inactive_users(180).await.unwrap();

    // Only user_inactive (id=200) should be banned
    assert_eq!(banned_user_ids, vec![200]);

    // Verify user_inactive is now banned
    assert!(pool.is_user_banned(200).await.unwrap());

    // Verify a warning was created
    let warnings = pool.find_user_warnings(200).await;
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].ban);
    assert_eq!(warnings[0].reason, "Inactivity: 180 days");
    assert_eq!(warnings[0].created_by_id, 1);
    assert!(warnings[0].expires_at.is_none());

    // Verify active user was NOT banned
    assert!(!pool.is_user_banned(201).await.unwrap());

    // Verify immune user was NOT banned
    assert!(!pool.is_user_banned(202).await.unwrap());
}

#[sqlx::test(fixtures("with_inactive_users"), migrations = "../storage/migrations")]
async fn test_ban_inactive_users_returns_zero_when_none_eligible(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // With a very high threshold, no users should be banned
    let banned_user_ids = pool.ban_inactive_users(9999).await.unwrap();
    assert!(banned_user_ids.is_empty());
}

#[sqlx::test(fixtures("with_inactive_users"), migrations = "../storage/migrations")]
async fn test_ban_inactive_users_skipped_when_setting_disabled(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // The default migration inserts inactive_user_ban_after_days = NULL (disabled)
    let settings = pool.get_arcadia_settings().await.unwrap();
    assert!(settings.inactive_user_ban_after_days.is_none());

    // Simulate what the periodic task does: skip when setting is None
    if let Some(days) = settings.inactive_user_ban_after_days {
        pool.ban_inactive_users(days).await.unwrap();
    }

    // No users should have been banned
    assert!(!pool.is_user_banned(200).await.unwrap());
    assert!(!pool.is_user_banned(201).await.unwrap());
    assert!(!pool.is_user_banned(202).await.unwrap());
}

#[sqlx::test(fixtures("with_inactive_users"), migrations = "../storage/migrations")]
async fn test_ban_inactive_users_is_idempotent(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let first_run = pool.ban_inactive_users(180).await.unwrap();
    assert_eq!(first_run, vec![200]);

    // Running again should ban nobody since user is already banned
    let second_run = pool.ban_inactive_users(180).await.unwrap();
    assert!(second_run.is_empty());
}

#[sqlx::test(fixtures("with_inactive_users"), migrations = "../storage/migrations")]
async fn test_ban_inactive_users_invalidates_their_tokens(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let mut settings = pool.get_arcadia_settings().await.unwrap();
    settings.inactive_user_ban_after_days = Some(180);
    pool.update_arcadia_settings(&settings).await.unwrap();
    let redis_pool = Arc::new(MockRedisPool::default());
    let token_issued_at = Utc::now().timestamp() - 1;

    inactive_users::ban_inactive_users(Arc::clone(&pool), Arc::clone(&redis_pool))
        .await
        .unwrap();

    let auth = Auth::new(redis_pool);
    assert!(auth.is_invalidated(200, token_issued_at).await.unwrap());
    assert!(!auth.is_invalidated(201, token_issued_at).await.unwrap());
}
