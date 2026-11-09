use arcadia_periodic_tasks::periodic_tasks::seeding_size::update_user_torrent_stats;
use arcadia_storage::connection_pool::ConnectionPool;
use sqlx::PgPool;
use std::borrow::Borrow;
use std::sync::Arc;

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_seeding_size"
))]
async fn test_user_torrent_stats(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    update_user_torrent_stats(Arc::clone(&pool)).await.unwrap();

    let pg_pool: &PgPool = (*pool).borrow();

    // User 100: seeds torrent 1 (100MB) + torrent 2 (200MB) = 300MB, leeches torrent 3
    // snatched torrent 1 and torrent 2
    let user_100: (i64, i32, i32, i32) = sqlx::query_as(
        "SELECT seeding_size, seeding, leeching, snatched FROM users WHERE id = 100",
    )
    .fetch_one(pg_pool)
    .await
    .unwrap();
    assert_eq!(user_100.0, 300000000); // seeding_size
    assert_eq!(user_100.1, 2); // seeding (2 distinct torrents)
    assert_eq!(user_100.2, 1); // leeching (1 torrent)
    assert_eq!(user_100.3, 2); // snatched (2 completed activities)

    // User 101: seeds torrent 1 with 3 peers, but counts only once = 100MB
    // snatched torrent 1
    let user_101: (i64, i32, i32, i32) = sqlx::query_as(
        "SELECT seeding_size, seeding, leeching, snatched FROM users WHERE id = 101",
    )
    .fetch_one(pg_pool)
    .await
    .unwrap();
    assert_eq!(user_101.0, 100000000); // seeding_size
    assert_eq!(user_101.1, 1); // seeding (1 distinct torrent)
    assert_eq!(user_101.2, 0); // leeching
    assert_eq!(user_101.3, 1); // snatched (1 completed activity)

    // User 102: no peers, no completed activities
    let user_102: (i64, i32, i32, i32) = sqlx::query_as(
        "SELECT seeding_size, seeding, leeching, snatched FROM users WHERE id = 102",
    )
    .fetch_one(pg_pool)
    .await
    .unwrap();
    assert_eq!(user_102.0, 0);
    assert_eq!(user_102.1, 0);
    assert_eq!(user_102.2, 0);
    assert_eq!(user_102.3, 0);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_seeding_size"
))]
async fn test_user_torrent_stats_reset_when_stopped(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let pg_pool: &PgPool = (*pool).borrow();

    // First update to set initial stats
    update_user_torrent_stats(Arc::clone(&pool)).await.unwrap();

    // Verify user 100 has stats set
    let user_100_before: (i64, i32, i32) =
        sqlx::query_as("SELECT seeding_size, seeding, leeching FROM users WHERE id = 100")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(user_100_before.0, 300000000);
    assert_eq!(user_100_before.1, 2);
    assert_eq!(user_100_before.2, 1);

    // Remove all peers for user 100 (simulating stopped seeding/leeching)
    sqlx::query("DELETE FROM peers WHERE user_id = 100")
        .execute(pg_pool)
        .await
        .unwrap();

    // Run update again
    update_user_torrent_stats(Arc::clone(&pool)).await.unwrap();

    // User 100 should now have seeding_size, seeding, leeching = 0
    // but snatched should remain (completed_at is permanent)
    let user_100_after: (i64, i32, i32, i32) = sqlx::query_as(
        "SELECT seeding_size, seeding, leeching, snatched FROM users WHERE id = 100",
    )
    .fetch_one(pg_pool)
    .await
    .unwrap();
    assert_eq!(user_100_after.0, 0);
    assert_eq!(user_100_after.1, 0);
    assert_eq!(user_100_after.2, 0);
    assert_eq!(user_100_after.3, 2); // snatched stays

    // User 101 should still have their stats
    let user_101: (i64, i32) =
        sqlx::query_as("SELECT seeding_size, seeding FROM users WHERE id = 101")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(user_101.0, 100000000);
    assert_eq!(user_101.1, 1);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_seeding_size"
))]
async fn test_user_torrent_stats_ignore_inactive_peers(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let pg_pool: &PgPool = (*pool).borrow();

    // User 100: mark the seeding peer on torrent 2 and the leeching peer as inactive
    sqlx::query("UPDATE peers SET active = false WHERE user_id = 100 AND torrent_id = 2")
        .execute(pg_pool)
        .await
        .unwrap();
    // User 101: mark 2 of the 3 peers on torrent 1 as inactive
    sqlx::query(
        "UPDATE peers SET active = false WHERE user_id = 101 AND peer_id != E'\\\\xc001000000000000000000000000000000000003'",
    )
    .execute(pg_pool)
    .await
    .unwrap();

    update_user_torrent_stats(Arc::clone(&pool)).await.unwrap();

    // User 100: only torrent 1 (100MB) is still actively seeded, no active leeching
    let user_100: (i64, i32, i32) =
        sqlx::query_as("SELECT seeding_size, seeding, leeching FROM users WHERE id = 100")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(user_100.0, 100000000);
    assert_eq!(user_100.1, 1);
    assert_eq!(user_100.2, 0);

    // User 101: one active peer left on torrent 1
    let user_101: (i64, i32) =
        sqlx::query_as("SELECT seeding_size, seeding FROM users WHERE id = 101")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(user_101.0, 100000000);
    assert_eq!(user_101.1, 1);

    // All peers of user 101 inactive: stats reset to 0
    sqlx::query("UPDATE peers SET active = false WHERE user_id = 101")
        .execute(pg_pool)
        .await
        .unwrap();

    update_user_torrent_stats(Arc::clone(&pool)).await.unwrap();

    let user_101: (i64, i32) =
        sqlx::query_as("SELECT seeding_size, seeding FROM users WHERE id = 101")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(user_101.0, 0);
    assert_eq!(user_101.1, 0);
}
