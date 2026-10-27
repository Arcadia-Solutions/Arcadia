use actix_web::web;
use arcadia_shared::tracker::models::{
    env::ArcadiaSettingsForTracker, infohash_2_id, passkey_2_id, torrent, user,
};
use arcadia_tracker::{config::Config, Tracker};
use parking_lot::{Mutex, RwLock};
use sqlx::PgPool;
use std::sync::OnceLock;

/// Only the keys that differ from their default are given. The database section is required
/// but unused: the tracker is given the test pool directly.
pub(crate) fn test_config() -> Config {
    yaml_serde::from_str(
        r#"
        database: { host: localhost, port: 5432, user: arcadia, password: password, name: arcadia }
        tracker:
          api_key: amazing_api_key_of_32_characters
          allowed_torrent_clients: [lt0F01-, qB, UTorrent]
          numwant_default: 50
          numwant_max: 200
          announce_max: 7200
          announce_min_enforced: 0 # Disable rate limiting for tests
          max_peers_per_torrent_per_user: 10
          flush_interval_milliseconds: 60000
          peer_expiry_interval: 600
          active_peer_ttl: 3600
          inactive_peer_ttl: 300
        "#,
    )
    .expect("valid test configuration")
}

/// Tracker with its stores loaded from the test database
pub(crate) async fn create_test_tracker(pool: PgPool, config: Config) -> web::Data<Tracker> {
    let settings = ArcadiaSettingsForTracker::from_database(&pool).await;
    let users = user::Map::from_database(&pool).await;
    let passkey2id = passkey_2_id::Map::from_database(&pool).await;
    let infohash2id = infohash_2_id::Map::from_database(&pool).await;
    let torrents = torrent::Map::from_database(&pool).await;

    web::Data::new(Tracker {
        config,
        pool,
        settings: RwLock::new(settings),
        metrics: OnceLock::new(),
        users: RwLock::new(users),
        passkey2id: RwLock::new(passkey2id),
        infohash2id: RwLock::new(infohash2id),
        torrents: Mutex::new(torrents),
        user_updates: Mutex::new(Default::default()),
        torrent_updates: Mutex::new(Default::default()),
        peer_updates: Mutex::new(Default::default()),
        announce_error_updates: Mutex::new(Default::default()),
        unpersisted_inactive_peers: Mutex::new(Vec::new()),
    })
}
