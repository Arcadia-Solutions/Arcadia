mod common;

use std::net::{IpAddr, Ipv4Addr};

use arcadia_shared::tracker::models::{
    peer::{self, Peer},
    peer_id::PeerId,
    torrent_update,
};
use arcadia_tracker::scheduler::reap;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

const ACTIVE_PEER_TTL: u64 = 7200;
const INACTIVE_PEER_TTL: u64 = 1_814_400;

/// The three peers of `with_test_peers`, all belonging to user 1 on torrent 1
const FRESH_SEEDER: PeerId = PeerId([0x11; 20]);
const STALE_SEEDER: PeerId = PeerId([0x33; 20]);
const DEAD_LEECHER: PeerId = PeerId([0x22; 20]);
/// Peers only added to the in-memory store, on torrent 2
const FRESH_LEECHER: PeerId = PeerId([0x44; 20]);
const STALE_LEECHER: PeerId = PeerId([0x55; 20]);
const DEAD_SEEDER: PeerId = PeerId([0x66; 20]);
/// Silent for longer than inactive_peer_ttl but never marked as inactive, like a peer loaded from
/// the database after the tracker was down for that long
const UNREAPED_DEAD_LEECHER: PeerId = PeerId([0x77; 20]);

fn index(peer_id: PeerId) -> peer::Index {
    peer::Index {
        user_id: 1,
        peer_id,
    }
}

fn peer(is_seeder: bool, is_active: bool, updated_at: DateTime<Utc>) -> Peer {
    Peer {
        ip_address: IpAddr::V4(Ipv4Addr::new(10, 10, 4, 89)),
        port: 24,
        is_seeder,
        is_active,
        has_sent_completed: false,
        updated_at,
        uploaded: 0,
        downloaded: 0,
    }
}

/// Peers that stopped announcing are marked as inactive, and once they have been silent for
/// longer than `inactive_peer_ttl` they are erased from the store and from the database.
///
/// Two torrents are used on purpose: expiration runs in parallel over the torrent store, so the
/// peers removed by each thread have to end up attributed to the right torrent.
#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_torrent_2",
        "with_test_peers"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_reap_expires_stale_peers(pool: PgPool) {
    // Torrent 2 gets its own dead peer in the database, so that a peer removed by one thread
    // cannot be attributed to the torrent of another one without the deletion missing it
    sqlx::query(
        r#"
        INSERT INTO peers (torrent_id, peer_id, ip, port, user_id, agent, uploaded, downloaded,
                           "left", seeder, active, created_at, updated_at)
        VALUES (2, $1, '10.10.4.92', 27, 1, 'test-agent/1.0', 0, 0, 0, true, false,
                NOW() - INTERVAL '30 days', NOW() - INTERVAL '30 days')
        "#,
    )
    .bind(DEAD_SEEDER.to_vec())
    .execute(&pool)
    .await
    .expect("could not insert the dead peer of torrent 2");

    let mut config = common::test_config();
    config.tracker.active_peer_ttl = ACTIVE_PEER_TTL;
    config.tracker.inactive_peer_ttl = INACTIVE_PEER_TTL;

    let now = Utc::now();
    // Silent for longer than active_peer_ttl, but not yet for inactive_peer_ttl
    let stale = now - Duration::seconds(ACTIVE_PEER_TTL as i64 + 60);

    // The database row of the stale seeder holds its last announce, like the store does
    sqlx::query("UPDATE peers SET updated_at = $1 WHERE peer_id = $2")
        .bind(stale)
        .bind(STALE_SEEDER.to_vec())
        .execute(&pool)
        .await
        .expect("could not age the stale seeder");

    let tracker = common::create_test_tracker(pool.clone(), config).await;
    // Silent for longer than inactive_peer_ttl
    let dead = now - Duration::seconds(INACTIVE_PEER_TTL as i64 + 60);

    {
        let mut torrents = tracker.torrents.lock();

        // Torrent 1 keeps the peers loaded from the database, with adjusted timestamps
        let torrent_1 = torrents.get_mut(&1).expect("torrent 1 is in the store");
        torrent_1
            .peers
            .insert(index(FRESH_SEEDER), peer(true, true, now));
        torrent_1
            .peers
            .insert(index(STALE_SEEDER), peer(true, true, stale));
        torrent_1
            .peers
            .insert(index(DEAD_LEECHER), peer(false, false, dead));
        torrent_1.seeders = 2;
        torrent_1.leechers = 0;

        // Torrent 2 keeps its dead peer loaded from the database
        let torrent_2 = torrents.get_mut(&2).expect("torrent 2 is in the store");
        torrent_2
            .peers
            .insert(index(FRESH_LEECHER), peer(false, true, now));
        torrent_2
            .peers
            .insert(index(STALE_LEECHER), peer(false, true, stale));
        torrent_2
            .peers
            .insert(index(UNREAPED_DEAD_LEECHER), peer(false, true, dead));
        assert!(
            torrent_2.peers.contains_key(&index(DEAD_SEEDER)),
            "the dead peer of torrent 2 is loaded from the database"
        );
        torrent_2.seeders = 0;
        torrent_2.leechers = 3;
    }

    {
        let mut users = tracker.users.write();
        let user = users.get_mut(&1).expect("user 1 is in the store");
        user.num_seeding = 2;
        user.num_leeching = 3;
    }

    let removed = reap(&tracker).await;

    assert_eq!(removed, 2, "one dead peer of each torrent is erased");

    // Scoped so the lock guards are dropped before the database query below
    {
        let torrents = tracker.torrents.lock();

        let torrent_1 = torrents.get(&1).unwrap();
        assert_eq!(torrent_1.peers.len(), 2, "the dead leecher is erased");
        assert!(
            torrent_1
                .peers
                .get(&index(FRESH_SEEDER))
                .expect("the fresh seeder is kept")
                .is_active,
            "a peer that announced recently stays active"
        );
        assert!(
            !torrent_1
                .peers
                .get(&index(STALE_SEEDER))
                .expect("the stale seeder is kept")
                .is_active,
            "a peer silent for longer than active_peer_ttl is marked as inactive"
        );
        assert!(torrent_1.peers.get(&index(DEAD_LEECHER)).is_none());
        assert_eq!(torrent_1.seeders, 1);
        assert_eq!(torrent_1.leechers, 0);

        let torrent_2 = torrents.get(&2).unwrap();
        assert_eq!(torrent_2.peers.len(), 3, "the dead seeder is erased");
        assert!(
            torrent_2
                .peers
                .get(&index(FRESH_LEECHER))
                .unwrap()
                .is_active
        );
        assert!(
            !torrent_2
                .peers
                .get(&index(STALE_LEECHER))
                .unwrap()
                .is_active
        );
        assert!(
            !torrent_2
                .peers
                .get(&index(UNREAPED_DEAD_LEECHER))
                .expect("a peer still marked as active is not erased before being counted out")
                .is_active,
            "a peer still marked as active is marked as inactive first, whatever its silence"
        );
        assert!(torrent_2.peers.get(&index(DEAD_SEEDER)).is_none());
        assert_eq!(torrent_2.seeders, 0);
        assert_eq!(torrent_2.leechers, 1);

        // Peer counts of the user are updated for the peers that got marked as inactive
        let users = tracker.users.read();
        let user = users.get(&1).unwrap();
        assert_eq!(user.num_seeding, 1);
        assert_eq!(user.num_leeching, 1);

        // Both torrents queue their own delta for the next flush
        let torrent_updates = tracker.torrent_updates.lock();
        let update_1 = torrent_updates
            .records
            .get(&torrent_update::Index { torrent_id: 1 })
            .expect("torrent 1 queued an update");
        assert_eq!(update_1.seeder_delta, -1);
        assert_eq!(update_1.leecher_delta, 0);
        let update_2 = torrent_updates
            .records
            .get(&torrent_update::Index { torrent_id: 2 })
            .expect("torrent 2 queued an update");
        assert_eq!(update_2.seeder_delta, 0);
        assert_eq!(update_2.leecher_delta, -2);
    }

    // Only the erased peers are deleted from the database, each from its own torrent, and the
    // ones marked as inactive are marked there too
    let remaining: Vec<(i32, Vec<u8>, bool)> = sqlx::query_as(
        "SELECT torrent_id, peer_id, active FROM peers ORDER BY torrent_id, peer_id",
    )
    .fetch_all(&pool)
    .await
    .expect("could not read the peers back");
    assert_eq!(
        remaining,
        vec![
            (1, FRESH_SEEDER.to_vec(), true),
            (1, STALE_SEEDER.to_vec(), false)
        ]
    );

    // Peers marked as inactive are kept until inactive_peer_ttl, so that what they report when
    // they announce again is counted from their previous announce. Only the dead peer that got
    // marked as inactive by the previous pass is erased now, without being counted out twice.
    assert_eq!(
        reap(&tracker).await,
        1,
        "inactive peers are not erased before inactive_peer_ttl"
    );
    let torrents = tracker.torrents.lock();
    let torrent_2 = torrents.get(&2).unwrap();
    assert!(torrent_2.peers.get(&index(UNREAPED_DEAD_LEECHER)).is_none());
    assert!(torrent_2.peers.get(&index(STALE_LEECHER)).is_some());
    assert_eq!(torrent_2.leechers, 1);
}

/// An announce can be flushed to the database between the moment reap marks the peer as inactive
/// in memory and the moment it persists it, for example by the flush on shutdown. That newer row
/// must stay active.
#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_peers"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_reap_keeps_newer_announce_active_in_database(pool: PgPool) {
    let mut config = common::test_config();
    config.tracker.active_peer_ttl = ACTIVE_PEER_TTL;
    config.tracker.inactive_peer_ttl = INACTIVE_PEER_TTL;

    let tracker = common::create_test_tracker(pool.clone(), config).await;

    // Stale in memory, while the database row (updated_at = NOW()) holds a newer announce
    let stale = Utc::now() - Duration::seconds(ACTIVE_PEER_TTL as i64 + 60);
    tracker
        .torrents
        .lock()
        .get_mut(&1)
        .expect("torrent 1 is in the store")
        .peers
        .insert(index(FRESH_SEEDER), peer(true, true, stale));

    reap(&tracker).await;

    let active: bool = sqlx::query_scalar("SELECT active FROM peers WHERE peer_id = $1")
        .bind(FRESH_SEEDER.to_vec())
        .fetch_one(&pool)
        .await
        .expect("could not read the peer back");
    assert!(active, "the newer announce is not overwritten as inactive");
}
