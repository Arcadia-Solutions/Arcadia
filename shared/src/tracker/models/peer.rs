use crate::error::Error;
use crate::tracker::models::peer_id::PeerId;
use crate::tracker::models::peer_update;
use chrono::serde::ts_seconds;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize, Serializer};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Map(IndexMap<Index, Peer>);

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
pub struct Index {
    pub user_id: u32,
    pub peer_id: PeerId,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Peer {
    pub ip_address: std::net::IpAddr,
    pub port: u16,
    pub is_seeder: bool,
    pub is_active: bool,
    // pub is_visible: bool,
    pub has_sent_completed: bool,
    #[serde(with = "ts_seconds")]
    pub updated_at: DateTime<Utc>,
    pub uploaded: u64,
    pub downloaded: u64,
}

impl Peer {
    /// Determines if the peer should be included in the peer list
    #[inline(always)]
    pub fn is_included_in_peer_list(&self) -> bool {
        self.is_active //&& self.is_visible
    }

    /// Determines if the peer should be included in the list of seeds
    #[inline(always)]
    pub fn is_included_in_seed_list(&self) -> bool {
        self.is_seeder && self.is_included_in_peer_list()
    }

    /// Determines if the peer should be included in the list of leeches
    #[inline(always)]
    pub fn is_included_in_leech_list(&self) -> bool {
        !self.is_seeder && self.is_included_in_peer_list()
    }
}

impl Map {
    pub fn new() -> Map {
        Map(IndexMap::new())
    }
}

impl Deref for Map {
    type Target = IndexMap<Index, Peer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.user_id, self.peer_id)
    }
}

impl Serialize for Index {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// we use peer_update::Index because we also need the torrent_id, which isn't in peer::Index
pub async fn remove_peers_from_database(db: &PgPool, peers: &Vec<peer_update::Index>) {
    if peers.is_empty() {
        return;
    }

    let mut user_ids: Vec<i32> = Vec::with_capacity(peers.len());
    let mut torrent_ids: Vec<i32> = Vec::with_capacity(peers.len());
    let mut peer_ids: Vec<Vec<u8>> = Vec::with_capacity(peers.len());

    for index in peers {
        user_ids.push(index.user_id as i32);
        torrent_ids.push(index.torrent_id as i32);
        peer_ids.push(index.peer_id.to_vec());
    }

    let result = sqlx::query!(
        r#"
                    DELETE FROM peers
                    WHERE (user_id, torrent_id, peer_id) IN (
                        SELECT t.user_id, t.torrent_id, t.peer_id
                        FROM (
                            SELECT * FROM unnest(
                                $1::int[],
                                $2::int[],
                                $3::bytea[]
                            ) AS t(user_id, torrent_id, peer_id)
                        ) AS t
                    )
                "#,
        &user_ids,
        &torrent_ids,
        &peer_ids
    )
    .execute(db)
    .await
    .map_err(|e| Error::DatabseError(e.to_string()));

    if result.is_err() {
        // TODO: reinsert the updates that failed and retry
        panic!("Failed removing peers from db: {}", result.err().unwrap());
    } else {
        log::info!("Removed {} peers", peers.len());
    }
}

/// Persists that the peers stopped announcing, the tracker keeps them in memory until they
/// announce again or get removed.
/// Rows updated after `active_cutoff` are left untouched, they come from an announce flushed
/// since the peers were marked as inactive in memory.
pub async fn mark_peers_inactive_in_database(
    db: &PgPool,
    peers: &Vec<peer_update::Index>,
    active_cutoff: DateTime<Utc>,
) -> Result<(), Error> {
    if peers.is_empty() {
        return Ok(());
    }

    let mut user_ids: Vec<i32> = Vec::with_capacity(peers.len());
    let mut torrent_ids: Vec<i32> = Vec::with_capacity(peers.len());
    let mut peer_ids: Vec<Vec<u8>> = Vec::with_capacity(peers.len());

    for index in peers {
        user_ids.push(index.user_id as i32);
        torrent_ids.push(index.torrent_id as i32);
        peer_ids.push(index.peer_id.to_vec());
    }

    let result = sqlx::query!(
        r#"
                    UPDATE peers
                    SET active = FALSE
                    WHERE (user_id, torrent_id, peer_id) IN (
                        SELECT t.user_id, t.torrent_id, t.peer_id
                        FROM (
                            SELECT * FROM unnest(
                                $1::int[],
                                $2::int[],
                                $3::bytea[]
                            ) AS t(user_id, torrent_id, peer_id)
                        ) AS t
                    )
                    -- stored as timestamp without time zone in UTC
                    AND updated_at < ($4::timestamptz AT TIME ZONE 'UTC')
                "#,
        &user_ids,
        &torrent_ids,
        &peer_ids,
        active_cutoff
    )
    .execute(db)
    .await
    .map_err(|e| Error::DatabseError(e.to_string()))?;

    log::info!("Marked {} peers as inactive", result.rows_affected());
    Ok(())
}
