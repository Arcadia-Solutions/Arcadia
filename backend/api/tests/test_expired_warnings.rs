use arcadia_storage::connection_pool::ConnectionPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_expired_warnings"))]
async fn test_clears_warned_for_user_with_only_expired_warnings(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let cleared_count = pool.clear_expired_warnings().await.unwrap();
    assert!(cleared_count >= 1);

    // User 300 had only an expired warning, should no longer be warned
    assert!(!pool.is_user_warned(300).await.unwrap());

    // Warning row should still exist
    let warnings = pool.find_user_warnings(300).await;
    assert_eq!(warnings.len(), 1);
}

#[sqlx::test(fixtures("with_expired_warnings"))]
async fn test_keeps_warned_for_user_with_active_warning(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    pool.clear_expired_warnings().await.unwrap();

    // User 301 has an active warning, should stay warned
    assert!(pool.is_user_warned(301).await.unwrap());
}

#[sqlx::test(fixtures("with_expired_warnings"))]
async fn test_keeps_warned_for_user_with_permanent_warning(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    pool.clear_expired_warnings().await.unwrap();

    // User 302 has a permanent warning (no expiry), should stay warned
    assert!(pool.is_user_warned(302).await.unwrap());
}

#[sqlx::test(fixtures("with_expired_warnings"))]
async fn test_unbans_user_with_only_expired_ban_warning(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    pool.clear_expired_warnings().await.unwrap();

    // User 303 had only an expired ban-warning, should be unbanned and unwarned
    assert!(!pool.is_user_banned(303).await.unwrap());
    assert!(!pool.is_user_warned(303).await.unwrap());

    // Warning row should still exist
    let warnings = pool.find_user_warnings(303).await;
    assert_eq!(warnings.len(), 1);
}

#[sqlx::test(fixtures("with_expired_warnings"))]
async fn test_keeps_ban_when_active_ban_warning_exists(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    pool.clear_expired_warnings().await.unwrap();

    // User 304 has an expired ban-warning AND an active ban-warning, should stay banned and warned
    assert!(pool.is_user_banned(304).await.unwrap());
    assert!(pool.is_user_warned(304).await.unwrap());
}

#[sqlx::test(fixtures("with_expired_warnings"))]
async fn test_keeps_warned_when_mixed_expired_and_active_warnings(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    pool.clear_expired_warnings().await.unwrap();

    // User 306 has an expired warning AND an active warning, should stay warned
    assert!(pool.is_user_warned(306).await.unwrap());

    // Both warning rows should still exist
    let warnings = pool.find_user_warnings(306).await;
    assert_eq!(warnings.len(), 2);
}

#[sqlx::test(fixtures("with_expired_warnings"))]
async fn test_is_idempotent(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let first_run = pool.clear_expired_warnings().await.unwrap();
    assert!(first_run >= 1);

    // Running again should clear nobody
    let second_run = pool.clear_expired_warnings().await.unwrap();
    assert_eq!(second_run, 0);
}
