use actix_web::web::Data;
use arcadia_shared::telemetry::{instrument_periodic_task, PeriodicTaskInstruments};
use arcadia_shared::tracker::models::{
    peer::{mark_peers_inactive_in_database, remove_peers_from_database},
    peer_update,
    torrent_update::{self, TorrentUpdate},
    Flushable,
};
use chrono::{Duration, Utc};
use std::convert::Infallible;
use std::sync::OnceLock;
use tokio::join;

use crate::Tracker;

static INSTRUMENTS: OnceLock<PeriodicTaskInstruments> = OnceLock::new();

fn instruments() -> &'static PeriodicTaskInstruments {
    INSTRUMENTS.get_or_init(|| PeriodicTaskInstruments::register("arcadia.tracker", "tracker_task"))
}

pub async fn handle(arc: &Data<Tracker>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(1));
    let mut counter = 0_u64;
    let _ = instruments();

    loop {
        interval.tick().await;
        counter += 1;

        if counter.is_multiple_of(arc.flush_interval_milliseconds) {
            flush(arc).await;
        }

        if counter.is_multiple_of(arc.peer_expiry_interval * 1000) {
            instrument_periodic_task::<_, _, Infallible>(instruments(), "reap", || async {
                Ok(reap(arc).await)
            })
            .await;
        }
    }
}

pub async fn flush(arc: &Data<Tracker>) {
    join!(
        instrument_periodic_task::<_, _, Infallible>(
            instruments(),
            "flush_user_updates",
            || async { Ok(arc.user_updates.flush_to_database(&arc.pool).await) },
        ),
        instrument_periodic_task::<_, _, Infallible>(
            instruments(),
            "flush_torrent_updates",
            || async { Ok(arc.torrent_updates.flush_to_database(&arc.pool).await) },
        ),
        instrument_periodic_task::<_, _, Infallible>(
            instruments(),
            "flush_peer_updates",
            || async { Ok(arc.peer_updates.flush_to_database(&arc.pool).await) },
        ),
        instrument_periodic_task::<_, _, Infallible>(
            instruments(),
            "flush_announce_error_updates",
            || async {
                Ok(arc
                    .announce_error_updates
                    .flush_to_database(&arc.pool)
                    .await)
            },
        )
    );
}

/// Remove peers that have not announced for some time
pub async fn reap(arc: &Data<Tracker>) -> u64 {
    let ttl = Duration::seconds(arc.active_peer_ttl.try_into().unwrap());
    let active_cutoff = Utc::now().checked_sub_signed(ttl).unwrap();
    let ttl = Duration::seconds(arc.inactive_peer_ttl.try_into().unwrap());
    let inactive_cutoff = Utc::now().checked_sub_signed(ttl).unwrap();
    let mut all_removed_peers: Vec<peer_update::Index> = Vec::new();
    let mut all_inactivated_peers: Vec<peer_update::Index> = Vec::new();

    for (torrent_id, torrent) in arc.torrents.lock().iter_mut() {
        let mut seeder_delta: i32 = 0;
        let mut leecher_delta: i32 = 0;

        // If a peer is marked as inactive and it has not announced for
        // more than inactive_peer_ttl, then it is permanently deleted.
        // Until then it is kept, so that its next announce is counted from
        // its previous one.
        let torrent_removed_peers = torrent
            .peers
            .extract_if(.., |_, peer| {
                !peer.is_active && peer.updated_at < inactive_cutoff
            })
            .map(|(index, _)| index)
            .collect::<Vec<arcadia_shared::tracker::models::peer::Index>>();
        all_removed_peers.extend(
            torrent_removed_peers
                .into_iter()
                .map(|index| peer_update::Index {
                    torrent_id: *torrent_id,
                    peer_id: index.peer_id,
                    user_id: index.user_id,
                })
                .collect::<Vec<peer_update::Index>>(),
        );

        for (index, peer) in torrent.peers.iter_mut() {
            // Peers get marked as inactive if not announced for more than
            // active_peer_ttl seconds. User peer count and torrent peer
            // count are updated to reflect.
            if peer.updated_at < active_cutoff && peer.is_active {
                peer.is_active = false;
                all_inactivated_peers.push(peer_update::Index {
                    torrent_id: *torrent_id,
                    peer_id: index.peer_id,
                    user_id: index.user_id,
                });

                arc.users.write().entry(index.user_id).and_modify(|user| {
                    if peer.is_seeder {
                        user.num_seeding = user.num_seeding.saturating_sub(1);
                    } else {
                        user.num_leeching = user.num_leeching.saturating_sub(1);
                    }
                });
                match peer.is_seeder {
                    true => seeder_delta -= 1,
                    false => leecher_delta -= 1,
                }
            }
        }

        // Update peer count of torrents and users
        if seeder_delta != 0 || leecher_delta != 0 {
            torrent.seeders = torrent.seeders.saturating_add_signed(seeder_delta);
            torrent.leechers = torrent.leechers.saturating_add_signed(leecher_delta);

            arc.torrent_updates.lock().upsert(
                torrent_update::Index {
                    torrent_id: torrent_id.to_owned(),
                },
                TorrentUpdate {
                    seeder_delta,
                    leecher_delta,
                    times_completed_delta: 0,
                },
            );
        }
    }

    // Retry the previous failures, unless the peer announced again or got removed since
    let previously_failed = std::mem::take(&mut *arc.unpersisted_inactive_peers.lock());
    {
        let torrents = arc.torrents.lock();
        all_inactivated_peers.extend(previously_failed.into_iter().filter(|index| {
            torrents
                .get(&index.torrent_id)
                .and_then(|torrent| {
                    torrent
                        .peers
                        .get(&arcadia_shared::tracker::models::peer::Index {
                            user_id: index.user_id,
                            peer_id: index.peer_id,
                        })
                })
                .is_some_and(|peer| !peer.is_active)
        }));
    }

    let removed_count = all_removed_peers.len() as u64;
    remove_peers_from_database(&arc.pool, &all_removed_peers).await;
    if let Err(error) =
        mark_peers_inactive_in_database(&arc.pool, &all_inactivated_peers, active_cutoff).await
    {
        log::error!("Failed marking peers as inactive in db, retrying on next reap: {error}");
        *arc.unpersisted_inactive_peers.lock() = all_inactivated_peers;
    }
    removed_count
}
