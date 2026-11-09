use arcadia_storage::connection_pool::ConnectionPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_artist",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_affiliated_artist",
    "with_artist_peer_stats"
))]
async fn test_update_artist_peer_stats_aggregates_affiliated_torrents(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let updated = pool.update_artist_peer_stats().await.unwrap();
    assert!(updated >= 1);

    let artist = pool.find_artist_by_id(1).await.unwrap();
    // Only torrent 1 is affiliated with artist 1 (torrent 2 is on an
    // unaffiliated title_group and torrent 100 is deleted).
    assert_eq!(artist.seeders_amount, 5);
    assert_eq!(artist.leechers_amount, 2);
    assert_eq!(artist.snatches_amount, 7);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_artist",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_affiliated_artist",
    "with_artist_peer_stats"
))]
async fn test_update_artist_peer_stats_is_idempotent(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let first_run = pool.update_artist_peer_stats().await.unwrap();
    assert!(first_run >= 1);

    // Running again with no underlying changes should update no rows.
    let second_run = pool.update_artist_peer_stats().await.unwrap();
    assert_eq!(second_run, 0);
}

#[sqlx::test(fixtures("with_test_users", "with_test_artist"))]
async fn test_update_artist_peer_stats_leaves_artist_without_torrents_at_zero(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    pool.update_artist_peer_stats().await.unwrap();

    let artist = pool.find_artist_by_id(1).await.unwrap();
    assert_eq!(artist.seeders_amount, 0);
    assert_eq!(artist.leechers_amount, 0);
    assert_eq!(artist.snatches_amount, 0);
}
