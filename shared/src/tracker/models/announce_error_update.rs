use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    error::Error,
    tracker::models::{Flushable, Mergeable, Queue, peer_id::PeerId},
};

/// Announce errors that are reported to the user, the other ones are either
/// client bugs or can't be attributed to a user
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "announce_error_code_enum", rename_all = "snake_case")]
pub enum AnnounceErrorCode {
    TorrentClientNotInWhitelist,
    InfoHashNotFound,
    TorrentIsDeleted,
    PeersPerTorrentPerUserLimit,
    SnatchLimitReached,
    InsufficientBonusPoints,
}

// Fields must be in same order as database primary key
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Index {
    pub user_id: u32,
    pub info_hash: [u8; 20],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceErrorUpdate {
    pub torrent_id: Option<u32>,
    pub error_code: AnnounceErrorCode,
    pub peer_id: PeerId,
    pub occurrences: u64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

impl Mergeable for AnnounceErrorUpdate {
    fn merge(&mut self, new: &Self) {
        if new.last_seen_at > self.last_seen_at {
            self.torrent_id = new.torrent_id;
            self.error_code = new.error_code;
            self.peer_id = new.peer_id;
            self.last_seen_at = new.last_seen_at;
        }

        self.first_seen_at = std::cmp::min(self.first_seen_at, new.first_seen_at);
        self.occurrences = self.occurrences.saturating_add(new.occurrences);
    }
}

impl Flushable<AnnounceErrorUpdate> for Mutex<Queue<Index, AnnounceErrorUpdate>> {
    async fn flush_to_database(&self, db: &PgPool) -> u64 {
        let amount_of_updates = self.lock().records.len();
        let updates = self
            .lock()
            .records
            .drain(0..amount_of_updates)
            .collect::<Vec<(Index, AnnounceErrorUpdate)>>();
        if updates.is_empty() {
            return 0;
        }

        let mut user_ids: Vec<i32> = Vec::with_capacity(updates.len());
        let mut info_hashes: Vec<Vec<u8>> = Vec::with_capacity(updates.len());
        let mut torrent_ids: Vec<Option<i32>> = Vec::with_capacity(updates.len());
        let mut error_codes: Vec<AnnounceErrorCode> = Vec::with_capacity(updates.len());
        let mut peer_ids: Vec<Vec<u8>> = Vec::with_capacity(updates.len());
        let mut occurrences: Vec<i64> = Vec::with_capacity(updates.len());
        let mut first_seen_ats: Vec<DateTime<Utc>> = Vec::with_capacity(updates.len());
        let mut last_seen_ats: Vec<DateTime<Utc>> = Vec::with_capacity(updates.len());

        for (index, update) in updates {
            user_ids.push(index.user_id as i32);
            info_hashes.push(index.info_hash.to_vec());
            torrent_ids.push(update.torrent_id.map(|torrent_id| torrent_id as i32));
            error_codes.push(update.error_code);
            peer_ids.push(update.peer_id.to_vec());
            occurrences.push(update.occurrences as i64);
            first_seen_ats.push(update.first_seen_at);
            last_seen_ats.push(update.last_seen_at);
        }

        // the join on users/torrents skips the rows whose user or torrent got deleted
        // since the error happened, which would otherwise make the whole insert fail
        let result = sqlx::query!(
            r#"
                INSERT INTO announce_errors (
                    user_id,
                    info_hash,
                    torrent_id,
                    error_code,
                    peer_id,
                    occurrences,
                    first_seen_at,
                    last_seen_at
                )
                SELECT
                    t.user_id,
                    t.info_hash,
                    torrents.id,
                    t.error_code,
                    t.peer_id,
                    t.occurrences,
                    t.first_seen_at,
                    t.last_seen_at
                FROM unnest(
                    $1::int[],
                    $2::bytea[],
                    $3::int[],
                    $4::announce_error_code_enum[],
                    $5::bytea[],
                    $6::bigint[],
                    $7::timestamptz[],
                    $8::timestamptz[]
                ) AS t(
                    user_id,
                    info_hash,
                    torrent_id,
                    error_code,
                    peer_id,
                    occurrences,
                    first_seen_at,
                    last_seen_at
                )
                JOIN users ON users.id = t.user_id
                LEFT JOIN torrents ON torrents.id = t.torrent_id
                ON CONFLICT (user_id, info_hash) DO UPDATE SET
                    torrent_id = EXCLUDED.torrent_id,
                    error_code = EXCLUDED.error_code,
                    peer_id = EXCLUDED.peer_id,
                    occurrences = announce_errors.occurrences + EXCLUDED.occurrences,
                    last_seen_at = EXCLUDED.last_seen_at
            "#,
            &user_ids,
            &info_hashes,
            &torrent_ids as &[Option<i32>],
            &error_codes as &[AnnounceErrorCode],
            &peer_ids,
            &occurrences,
            &first_seen_ats,
            &last_seen_ats
        )
        .execute(db)
        .await
        .map_err(|e| Error::DatabseError(e.to_string()));

        match result {
            Ok(query_result) => {
                log::info!("Inserted {amount_of_updates} announce error updates");
                query_result.rows_affected()
            }
            Err(error) => {
                log::error!("Failed to insert announce error updates: {error}");
                0
            }
        }
    }
}
