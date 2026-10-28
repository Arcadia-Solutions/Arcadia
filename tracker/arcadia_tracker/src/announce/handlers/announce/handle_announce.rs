use std::{
    future::{self, Ready},
    net::IpAddr,
    str::FromStr,
};

use crate::{
    announce::{
        error::{AnnounceError, Result},
        models::{
            announce::{Announce, AnnounceEvent},
            warning::{AnnounceWarning, WarningCollection},
        },
    },
    services::announce_service::{check_and_deduct_snatch_cost, is_torrent_client_allowed},
    Tracker,
};
use actix_web::{
    dev,
    web::{Data, Path},
    FromRequest, HttpRequest, HttpResponse,
};
use arcadia_shared::tracker::models::{
    announce_error_update::{self, AnnounceErrorUpdate},
    peer::{self, Peer},
    peer_id::PeerId,
    peer_update::{self, PeerUpdate},
    torrent::InfoHash,
    torrent_update::{self, TorrentUpdate},
    user::Passkey,
    user_update::{self, UserUpdate},
};
use chrono::{Duration, Utc};
use log::debug;
use opentelemetry::KeyValue;
use rand::{rng, seq::IteratorRandom, RngExt};

#[derive(Debug)]
pub struct UserAgent(pub String);

impl std::ops::Deref for UserAgent {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl FromRequest for UserAgent {
    type Error = AnnounceError;
    type Future = Ready<std::result::Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let agent = req
            .headers()
            .get("User-Agent")
            .ok_or(AnnounceError::NoUserAgent)
            .and_then(|s| {
                let agent_string = s.to_str()?;
                if agent_string.len() > 64 {
                    // Block user agent strings that are too long. (For Database reasons)
                    Err(AnnounceError::UserAgentTooLong)
                } else {
                    Ok(UserAgent(String::from(agent_string)))
                }
            });

        future::ready(agent)
    }
}

pub struct ClientIp(pub std::net::IpAddr);

impl FromRequest for ClientIp {
    type Error = AnnounceError;
    type Future = Ready<std::result::Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let tracker = req.app_data::<Data<Tracker>>().expect("app data set");

        let header_name_opt = &tracker.reverse_proxy_client_ip_header_name;

        let ip_result = if let Some(header_name) = header_name_opt {
            req.headers()
                .get(header_name)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| {
                    s.split(',')
                        .next_back()
                        .and_then(|s| s.trim().parse::<IpAddr>().ok())
                })
                .map(ClientIp)
                .ok_or(AnnounceError::InternalTrackerError)
        } else {
            req.peer_addr()
                .map(|addr| ClientIp(addr.ip()))
                .ok_or(AnnounceError::InternalTrackerError)
        };

        future::ready(ip_result)
    }
}

#[utoipa::path(
    get,
    operation_id = "Announce",
    tag = "Announce",
    path = "/announce/{passkey}",
    responses(
        (status = 200, description = "Announce"),
    )
)]
pub async fn exec(
    arc: Data<Tracker>,
    passkey: Path<String>,
    user_agent: UserAgent,
    ann: Announce,
    ClientIp(client_ip): ClientIp,
) -> Result<HttpResponse> {
    let info_hash = ann.info_hash;
    let peer_id = ann.peer_id;
    let result = handle(arc.clone(), &passkey, user_agent, ann, client_ip).await;

    if let Some(m) = arc.metrics.get() {
        match &result {
            Ok(_) => m.announces_ok.add(1, &[]),
            Err(e) => m
                .announces_err
                .add(1, &[KeyValue::new("error", e.as_ref().to_string())]),
        }
    }

    if let Err(error) = &result {
        record_announce_error(&arc, error, &passkey, info_hash, peer_id);
    }

    result
}

/// Upper bound of announce errors waiting to be flushed, so that a client
/// announcing lots of unknown info_hashes can't fill up the memory
const MAX_PENDING_ANNOUNCE_ERRORS: usize = 100_000;

/// Queues the error so that the user can see it on the site
fn record_announce_error(
    arc: &Tracker,
    error: &AnnounceError,
    passkey: &str,
    info_hash: InfoHash,
    peer_id: PeerId,
) {
    let Some(error_code) = error.reported_error_code() else {
        return;
    };
    let Some(user_id) = Passkey::from_str(passkey)
        .ok()
        .and_then(|passkey| arc.passkey2id.read().get(&passkey).cloned())
    else {
        return;
    };
    let torrent_id = arc.infohash2id.read().get(&info_hash).cloned();

    let mut announce_error_updates = arc.announce_error_updates.lock();
    let index = announce_error_update::Index {
        user_id,
        info_hash: info_hash.0,
    };
    if announce_error_updates.records.len() >= MAX_PENDING_ANNOUNCE_ERRORS
        && !announce_error_updates.records.contains_key(&index)
    {
        return;
    }
    let now = Utc::now();
    announce_error_updates.upsert(
        index,
        AnnounceErrorUpdate {
            torrent_id,
            error_code,
            peer_id,
            occurrences: 1,
            first_seen_at: now,
            last_seen_at: now,
        },
    );
}

async fn handle(
    arc: Data<Tracker>,
    passkey: &str,
    user_agent: UserAgent,
    ann: Announce,
    client_ip: IpAddr,
) -> Result<HttpResponse> {
    // let headers = req.headers();
    // if headers.contains_key(ACCEPT_LANGUAGE)
    //     || headers.contains_key(REFERER)
    //     || headers.contains_key(ACCEPT_CHARSET)
    //     // This header check may block Non-bittorrent client `Aria2` to access tracker,
    //     // Because they always add this header which other clients don't have.
    //     //
    //     // See: https://blog.rhilip.info/archives/1010/ ( in Chinese )
    //     || headers.contains_key("want-digest")
    // {
    //     return Err(AnnounceError::AbnormalAccess);
    // }

    if !is_torrent_client_allowed(&ann.peer_id, &arc.allowed_torrent_clients.clients) {
        return Err(AnnounceError::TorrentClientNotInWhitelist);
    }

    let passkey = Passkey::from_str(passkey).or(Err(AnnounceError::InvalidPasskey))?;
    // Validate passkey
    let user_id = arc
        .passkey2id
        .read()
        .get(&passkey)
        .cloned()
        .ok_or(AnnounceError::UserNotFound)?;

    // Validate torrent
    let torrent_id_res = arc
        .infohash2id
        .read()
        .get(&ann.info_hash)
        .ok_or(AnnounceError::InfoHashNotFound)
        .cloned();

    // if let Ok(user) = &user {
    //     if let Err(InfoHashNotFound) = torrent_id_res {
    //         tracker.unregistered_info_hash_updates.lock().upsert(
    //             unregistered_info_hash_update::Index {
    //                 user_id: user.id,
    //                 info_hash: queries.info_hash,
    //             },
    //             UnregisteredInfoHashUpdate {
    //                 created_at: now,
    //                 updated_at: now,
    //             },
    //         );
    //     }
    // }

    let torrent_id = torrent_id_res?;

    let mut warnings = WarningCollection::new();

    let now = Utc::now();

    // check and deduct bonus points snatch cost for new leeches BEFORE acquiring the main lock
    // this avoids holding the lock across an async database call
    if ann.event != AnnounceEvent::Stopped && ann.left != 0 {
        let is_new_peer = {
            let torrent_guard = arc.torrents.lock();
            let torrent = torrent_guard
                .get(&torrent_id)
                .ok_or(AnnounceError::TorrentNotFound)?;

            if torrent.is_deleted {
                return Err(AnnounceError::TorrentIsDeleted);
            }

            !torrent.peers.contains_key(&peer::Index {
                user_id,
                peer_id: ann.peer_id,
            })
        };

        if is_new_peer {
            let (transfer_to, charge_on_resnatch, snatch_cost_factor, snatch_reward_factor) = {
                let settings = arc.settings.read();
                (
                    settings
                        .snatched_torrent_bonus_points_transferred_to
                        .clone(),
                    settings.charge_bonus_points_on_resnatch,
                    settings.global_snatch_bonus_points_cost_factor,
                    settings.global_snatch_bonus_points_reward_factor,
                )
            };
            check_and_deduct_snatch_cost(
                &arc.pool,
                torrent_id,
                user_id,
                transfer_to.as_ref(),
                charge_on_resnatch,
                snatch_cost_factor,
                snatch_reward_factor,
            )
            .await?;
        }
    }

    let (
        upload_factor,
        download_factor,
        uploaded_delta,
        downloaded_delta,
        seeder_delta,
        leecher_delta,
        times_completed_delta,
        // is_visible,
        // is_active_after_stop,
        // user,
        user_id,
        has_requested_seed_list,
        has_requested_leech_list,
        response,
    ) = {
        let mut torrent_guard = arc.torrents.lock();
        let torrent = torrent_guard
            .get_mut(&torrent_id)
            .ok_or(AnnounceError::TorrentNotFound)?;

        if torrent.is_deleted {
            return Err(AnnounceError::TorrentIsDeleted);
        }

        // Change of upload/download compared to previous announce
        let uploaded_delta;
        let downloaded_delta;
        let seeder_delta;
        let leecher_delta;
        let times_completed_delta;
        // let is_visible;
        // let mut is_active_after_stop = false;

        if ann.event == AnnounceEvent::Stopped {
            // Try and remove the peer
            let removed_peer = torrent.peers.swap_remove(&peer::Index {
                user_id,
                peer_id: ann.peer_id,
            });
            // Check if peer was removed
            if let Some(peer) = removed_peer {
                // Calculate change in upload and download compared to previous
                // announce
                uploaded_delta = ann.uploaded.saturating_sub(peer.uploaded);
                downloaded_delta = ann.downloaded.saturating_sub(peer.downloaded);

                leecher_delta = 0 - peer.is_included_in_leech_list() as i32;
                seeder_delta = 0 - peer.is_included_in_seed_list() as i32;

                for (&index, &peer) in torrent.peers.iter() {
                    if index.user_id == user_id && peer.is_active {
                        // is_active_after_stop = true;

                        break;
                    }
                }
            } else {
                // Some clients (namely transmission) will keep sending
                // `stopped` events until a successful announce is received.
                // If a user's network is having issues, their peer might be
                // deleted for inactivity from missed announces. If their peer
                // isn't found when we receive a `stopped` event from them
                // after regaining network connectivity, we can't return an
                // error otherwise the client might enter into an infinite loop
                // of sending `stopped` events. To prevent this, we need to
                // send a warning (i.e. succcessful announce) instead, so that
                // the client can successfully restart its session.
                leecher_delta = 0;
                seeder_delta = 0;
                uploaded_delta = 0;
                downloaded_delta = 0;
            }

            times_completed_delta = 0;
            // is_visible = false;
        } else {
            // Insert the peer into the in-memory db
            let mut old_peer: Option<Peer> = None;
            let new_peer = *torrent
                .peers
                .entry(peer::Index {
                    user_id,
                    peer_id: ann.peer_id,
                })
                .and_modify(|peer| {
                    old_peer = Some(*peer);

                    peer.ip_address = client_ip;
                    peer.port = ann.port;
                    peer.is_seeder = ann.left == 0;
                    // peer.is_visible = peer.is_included_in_leech_list();
                    peer.is_active = true;
                    peer.has_sent_completed =
                        peer.has_sent_completed || ann.event == AnnounceEvent::Completed;
                    peer.updated_at = now;
                    peer.uploaded = ann.uploaded;
                    peer.downloaded = ann.downloaded;
                })
                .or_insert(peer::Peer {
                    ip_address: client_ip,
                    port: ann.port,
                    is_seeder: ann.left == 0,
                    is_active: true,
                    // is_visible: true,
                    has_sent_completed: ann.event == AnnounceEvent::Completed,
                    updated_at: now,
                    uploaded: ann.uploaded,
                    downloaded: ann.downloaded,
                });

            // is_visible = new_peer.is_visible;

            // Update the user and torrent seeding/leeching counts in the
            // in-memory db
            match old_peer {
                Some(old_peer) => {
                    leecher_delta = new_peer.is_included_in_leech_list() as i32
                        - old_peer.is_included_in_leech_list() as i32;
                    seeder_delta = new_peer.is_included_in_seed_list() as i32
                        - old_peer.is_included_in_seed_list() as i32;
                    times_completed_delta = (new_peer.is_seeder && !old_peer.is_seeder) as u32;

                    // Calculate change in upload and download compared to previous
                    // announce
                    if ann.uploaded < old_peer.uploaded || ann.downloaded < old_peer.downloaded {
                        // Client sent the same peer id but restarted the session
                        // Assume delta is 0
                        uploaded_delta = 0;
                        downloaded_delta = 0;
                    } else {
                        // Assume client continues previously tracked session
                        uploaded_delta = ann.uploaded - old_peer.uploaded;
                        downloaded_delta = ann.downloaded - old_peer.downloaded;
                    }

                    // Warn user if peer last announced less than
                    // announce_min_enforced seconds ago and it's
                    // not their first completed event
                    if old_peer
                        .updated_at
                        .checked_add_signed(Duration::seconds(arc.announce_min_enforced.into()))
                        .is_some_and(|blocked_until| blocked_until > now)
                        && (ann.event != AnnounceEvent::Completed || old_peer.has_sent_completed)
                    {
                        log::info!(
                            "Rate limit warning for user {} on torrent {}",
                            user_id,
                            torrent_id
                        );
                        warnings.add(AnnounceWarning::RateLimitExceeded);
                    }
                }
                None => {
                    // new peer is inserted

                    // Make sure user is only allowed N peers per torrent.
                    let mut peer_count = 0;

                    for (&index, &peer) in torrent.peers.iter() {
                        if index.user_id == user_id && peer.is_active {
                            peer_count += 1;

                            if peer_count > arc.max_peers_per_torrent_per_user {
                                torrent.peers.swap_remove(&peer::Index {
                                    user_id,
                                    peer_id: ann.peer_id,
                                });

                                return Err(AnnounceError::PeersPerTorrentPerUserLimit(
                                    arc.max_peers_per_torrent_per_user,
                                ));
                            }
                        }
                    }

                    // Check daily snatch limit for new leeches
                    if !new_peer.is_seeder {
                        let now_ts = now.timestamp();
                        let cutoff = now_ts - 86400; // 24h in seconds
                        if let Some(user) = arc.users.write().get_mut(&user_id)
                            && let Some(max) = user.max_snatches_per_day
                        {
                            // Clean up torrents that started being leeched more than 24 hours ago
                            user.recent_leeches.retain(|&(_, ts)| ts > cutoff);
                            // Check if there is already a record of this user leeching this torrent
                            // in the past 24h
                            let leeched_recently = user
                                .recent_leeches
                                .iter()
                                .any(|&(tid, _)| tid == torrent_id);
                            if !leeched_recently {
                                if user.recent_leeches.len() >= max as usize {
                                    torrent.peers.swap_remove(&peer::Index {
                                        user_id,
                                        peer_id: ann.peer_id,
                                    });
                                    return Err(AnnounceError::SnatchLimitReached(max));
                                }
                                user.recent_leeches.push((torrent_id, now_ts));
                            }
                        }
                    }

                    leecher_delta = new_peer.is_included_in_leech_list() as i32;
                    seeder_delta = new_peer.is_included_in_seed_list() as i32;
                    times_completed_delta = 0;

                    // Since this is a new peer, they have no upload or download history
                    uploaded_delta = 0;
                    downloaded_delta = 0;
                }
            }
        }

        // Has to be adjusted before the peer list is generated
        torrent.seeders = torrent.seeders.saturating_add_signed(seeder_delta);
        torrent.leechers = torrent.leechers.saturating_add_signed(leecher_delta);
        torrent.times_completed = torrent
            .times_completed
            .saturating_add(times_completed_delta);

        // Generate peer lists to return to client
        let mut peers_ipv4: Vec<u8> = Vec::new();
        let mut peers_ipv6: Vec<u8> = Vec::new();

        let mut has_requested_seed_list = false;
        let mut has_requested_leech_list = false;

        // Only provide peer list if
        // - it is not a stopped event,
        // - there exist leechers (we have to remember to update the torrent leecher count before this check)
        // - there is no warning in the response
        if ann.event != AnnounceEvent::Stopped && torrent.leechers > 0 && warnings.is_empty() {
            let mut peers: Vec<(&peer::Index, &Peer)> = Vec::with_capacity(std::cmp::min(
                ann.numwant,
                torrent.seeders as usize + torrent.leechers as usize,
            ));

            // Don't return peers with the same user id or those that are marked as inactive
            let valid_peers = torrent.peers.iter().filter(|(index, peer)| {
                index.user_id != user_id && peer.is_included_in_peer_list()
            });

            // Make sure leech peer lists are filled with seeds
            if ann.left > 0 && torrent.seeders > 0 && ann.numwant > peers.len() {
                has_requested_seed_list = true;
                peers.extend(
                    valid_peers
                        .clone()
                        .filter(|(_, peer)| peer.is_seeder)
                        .sample(&mut rng(), ann.numwant),
                );
            }

            // Otherwise only send leeches until the numwant is reached
            if torrent.leechers > 0 && ann.numwant > peers.len() {
                has_requested_leech_list = true;
                peers.extend(
                    valid_peers
                        .filter(|(_, peer)| !peer.is_seeder)
                        .sample(&mut rng(), ann.numwant.saturating_sub(peers.len())),
                );
            }

            // Split peers into ipv4 and ipv6 variants and serialize their socket
            // to bytes according to the bittorrent spec
            for (_, peer) in peers.iter() {
                match peer.ip_address {
                    IpAddr::V4(ip) => {
                        peers_ipv4.extend(&ip.octets());
                        peers_ipv4.extend(&peer.port.to_be_bytes());
                    }
                    IpAddr::V6(ip) => {
                        peers_ipv6.extend(&ip.octets());
                        peers_ipv6.extend(&peer.port.to_be_bytes());
                    }
                }
            }
        }

        // Write out bencoded response (keys must be sorted to be within spec)
        let mut response: Vec<u8> = Vec::with_capacity(
            82 // literal characters
            + 5 * 5 // numbers with estimated digit quantity for each
            + peers_ipv4.len() * 6 + 5 // bytes per ipv4 plus estimated length prefix
            + peers_ipv6.len() * 18 + 5 // bytes per ipv6 plus estimated length prefix
            + warnings.max_byte_length(), // max bytes per warning message plus separator
        );

        response.extend(b"d8:completei");
        if !warnings.is_empty() {
            response.extend(b"0")
        } else {
            response.extend(torrent.seeders.to_string().as_bytes());
        }
        response.extend(b"e10:downloadedi");
        response.extend(torrent.times_completed.to_string().as_bytes());
        response.extend(b"e10:incompletei");
        if !warnings.is_empty() {
            response.extend(b"0");
        } else {
            response.extend(torrent.leechers.to_string().as_bytes());
        }

        response.extend(b"e8:intervali");
        let interval = rng().random_range(arc.announce_min..=arc.announce_max);
        response.extend(interval.to_string().as_bytes());
        response.extend(b"e12:min intervali");
        response.extend(arc.announce_min.to_string().as_bytes());
        response.extend(b"e5:peers");

        if peers_ipv4.is_empty() {
            response.extend(b"0:")
        } else {
            response.extend(peers_ipv4.len().to_string().as_bytes());
            response.extend(b":");
            response.extend(&peers_ipv4);
        }

        if !peers_ipv6.is_empty() {
            response.extend(b"6:peers6");
            response.extend(peers_ipv6.len().to_string().as_bytes());
            response.extend(b":");
            response.extend(peers_ipv6);
        }

        if let Some(warning_message) = warnings.into_message() {
            response.extend(b"15:warning message");
            response.extend(warning_message.len().to_string().as_bytes());
            response.extend(b":");
            response.extend(warning_message);
        }

        response.extend(b"e");

        let upload_factor = std::cmp::max(
            arc.settings.read().global_upload_factor,
            torrent.upload_factor,
        );
        let download_factor = std::cmp::min(
            arc.settings.read().global_download_factor,
            torrent.download_factor,
        );

        // Has to be dropped before any `await` calls.
        //
        // Unfortunately, `Drop` currently doesn't work in rust with borrowed values
        // so we have to use a giant scope instead.
        //
        // See:
        // - https://github.com/rust-lang/rust/issues/57478
        // - https://stackoverflow.com/questions/73519148/why-does-send-value-that-is-dropd-before-await-mean-the-future-is-send
        // - https://github.com/rust-lang/rust/issues/101135
        //    - Once this issue is fixed, we can remove the scope and rely solely on `Drop`.
        drop(torrent_guard);

        (
            upload_factor,
            download_factor,
            uploaded_delta,
            downloaded_delta,
            seeder_delta,
            leecher_delta,
            times_completed_delta,
            // is_visible,
            // is_active_after_stop,
            // user,
            user_id,
            has_requested_seed_list,
            has_requested_leech_list,
            response,
        )
    };

    let credited_uploaded_delta = upload_factor as u64 * uploaded_delta / 100;
    let credited_downloaded_delta = download_factor as u64 * downloaded_delta / 100;

    let completed_at = if ann.event == AnnounceEvent::Completed {
        Some(now)
    } else {
        None
    };

    if seeder_delta != 0
        || leecher_delta != 0
        || has_requested_seed_list
        || has_requested_leech_list
    {
        arc.users.write().entry(user_id).and_modify(|user| {
            user.num_seeding = user.num_seeding.saturating_add_signed(seeder_delta);
            user.num_leeching = user.num_leeching.saturating_add_signed(leecher_delta);

            // TODO: setup seed/leech lists getting rate limiting
            // has been partially done in unit3d-announce
            // if has_requested_seed_list {
            //     user.receive_seed_list_rates.tick();
            // }

            // if has_requested_leech_list {
            //     user.receive_leech_list_rates.tick();
            // }
        });
    }

    arc.peer_updates.lock().upsert(
        peer_update::Index {
            peer_id: ann.peer_id,
            torrent_id,
            user_id,
        },
        PeerUpdate {
            ip: client_ip,
            port: ann.port,
            agent: user_agent.0,
            uploaded: ann.uploaded,
            downloaded: ann.downloaded,
            is_active: ann.event != AnnounceEvent::Stopped,
            is_seeder: ann.left == 0,
            left: ann.left,
            created_at: now,
            updated_at: now,
            // Activity fields for torrent_activities table
            completed_at,
            uploaded_delta: credited_uploaded_delta,
            downloaded_delta: credited_downloaded_delta,
            real_uploaded_delta: uploaded_delta,
            real_downloaded_delta: downloaded_delta,
        },
    );

    if uploaded_delta != 0 || downloaded_delta != 0 {
        arc.user_updates.lock().upsert(
            user_update::Index { user_id },
            UserUpdate {
                uploaded_delta: credited_uploaded_delta,
                downloaded_delta: credited_downloaded_delta,
                real_uploaded_delta: uploaded_delta,
                real_downloaded_delta: downloaded_delta,
            },
        );
    }

    if seeder_delta != 0
        || leecher_delta != 0
        || times_completed_delta != 0
        || uploaded_delta != 0
        || downloaded_delta != 0
    {
        arc.torrent_updates.lock().upsert(
            torrent_update::Index { torrent_id },
            TorrentUpdate {
                seeder_delta,
                leecher_delta,
                times_completed_delta,
            },
        );
    }
    debug!(
        "Announce response: {:?}",
        String::from_utf8_lossy(&response)
    );

    Ok(HttpResponse::Ok().body(response))
}
