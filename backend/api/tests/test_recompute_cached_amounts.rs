use arcadia_storage::connection_pool::ConnectionPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_artist",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_affiliated_artist",
        "with_artist_peer_stats"
    ),
    migrations = "../storage/migrations"
)]
async fn test_recompute_cached_amounts_recomputes_artist_stats(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    pool.recompute_cached_amounts().await.unwrap();

    let artist = pool.find_artist_by_id(1).await.unwrap();
    // Only torrent 1 is affiliated with artist 1 (torrent 2 is on an
    // unaffiliated title_group and torrent 100 is deleted).
    assert_eq!(artist.seeders_amount, 5);
    assert_eq!(artist.leechers_amount, 2);
    assert_eq!(artist.snatches_amount, 7);
}
