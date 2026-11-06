use arcadia_storage::connection_pool::ConnectionPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_torrent_clients"
    ),
    migrations = "../storage/migrations"
)]
async fn test_get_user_torrent_clients(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let clients = pool.get_user_torrent_clients(100).await.unwrap();

    // User 100 should have exactly 2 distinct clients
    assert_eq!(clients.len(), 2);

    // Clients are ordered by agent, ip, port
    // First client: qBittorrent (aggregated from 2 peer entries)
    let qbittorrent_client = &clients[0];
    assert_eq!(qbittorrent_client.agent, "qBittorrent/4.5.0");
    assert_eq!(qbittorrent_client.ip.ip().to_string(), "192.168.1.100");
    assert_eq!(qbittorrent_client.port, 6881);
    // Aggregated: 1000000 + 2000000 = 3000000
    assert_eq!(qbittorrent_client.real_uploaded, 3000000);
    // Aggregated: 500000 + 300000 = 800000
    assert_eq!(qbittorrent_client.real_downloaded, 800000);
    // first_seen_at should be MIN(created_at) = 2025-01-01 10:00:00
    assert_eq!(
        qbittorrent_client
            .first_seen_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        "2025-01-01 10:00:00"
    );
    // last_seen_at should be MAX(updated_at) = 2025-01-20 14:00:00
    assert_eq!(
        qbittorrent_client
            .last_seen_at
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        "2025-01-20 14:00:00"
    );

    // Second client: Transmission
    let transmission_client = &clients[1];
    assert_eq!(transmission_client.agent, "Transmission/3.0");
    assert_eq!(transmission_client.ip.ip().to_string(), "192.168.1.100");
    assert_eq!(transmission_client.port, 51413);
    assert_eq!(transmission_client.real_uploaded, 500000);
    assert_eq!(transmission_client.real_downloaded, 100000);

    // Verify user 101's client is not included
    let user_101_clients = pool.get_user_torrent_clients(101).await.unwrap();
    assert_eq!(user_101_clients.len(), 1);
    assert_eq!(user_101_clients[0].agent, "Deluge/2.1.1");
}
