use arcadia_shared::tracker::models::announce_error_update::AnnounceErrorCode;
use arcadia_storage::connection_pool::ConnectionPool;
use sqlx::PgPool;

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_announce_errors"
))]
async fn test_removes_resolved_and_stale_announce_errors(pool: PgPool) {
    let pool = ConnectionPool::with_pg_pool(pool);

    let removed_count = pool
        .remove_resolved_and_stale_announce_errors(86400)
        .await
        .unwrap();
    assert_eq!(removed_count, 2);

    let user_100_notifications = pool.find_all_notifications(100, false).await.unwrap();
    assert!(user_100_notifications.announce_errors.is_empty());

    let user_101_notifications = pool.find_all_notifications(101, false).await.unwrap();
    assert_eq!(user_101_notifications.announce_errors.len(), 1);
    let announce_error = &user_101_notifications.announce_errors[0];
    assert_eq!(announce_error.torrent_id, Some(1));
    assert_eq!(
        announce_error.error_code,
        AnnounceErrorCode::PeersPerTorrentPerUserLimit
    );
    assert!(announce_error.title_group_name.is_some());

    let user_101_counts = pool.find_notification_counts(101).await.unwrap();
    assert_eq!(user_101_counts.announce_errors, 1);
}
