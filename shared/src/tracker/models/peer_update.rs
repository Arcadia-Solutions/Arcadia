use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::ipnetwork::IpNetwork};

use crate::{
    error::Error,
    tracker::models::{Flushable, Mergeable, Queue, peer_id::PeerId},
};

// Fields must be in same order as database primary key
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Index {
    pub user_id: u32,
    pub torrent_id: u32,
    pub peer_id: PeerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerUpdate {
    pub ip: std::net::IpAddr,
    pub port: u16,
    pub agent: String,
    pub uploaded: u64,
    pub downloaded: u64,
    pub is_active: bool,
    pub is_seeder: bool,
    pub left: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // fields for torrent_activities update
    // total_seed_time is updated by the backend's scheduler
    pub completed_at: Option<DateTime<Utc>>,
    pub uploaded_delta: u64,
    pub downloaded_delta: u64,
    pub real_uploaded_delta: u64,
    pub real_downloaded_delta: u64,
}

impl Mergeable for PeerUpdate {
    fn merge(&mut self, new: &Self) {
        if new.updated_at > self.updated_at {
            self.ip = new.ip;
            self.port = new.port;
            self.agent = new.agent.clone();
            self.uploaded = new.uploaded;
            self.downloaded = new.downloaded;
            self.is_active = new.is_active;
            self.is_seeder = new.is_seeder;
            self.left = new.left;
            self.updated_at = new.updated_at;
        }

        self.created_at = std::cmp::min(self.created_at, new.created_at);

        if self.completed_at.is_none() && new.completed_at.is_some() {
            self.completed_at = new.completed_at;
        }
        self.uploaded_delta = self.uploaded_delta.saturating_add(new.uploaded_delta);
        self.downloaded_delta = self.downloaded_delta.saturating_add(new.downloaded_delta);
        self.real_uploaded_delta = self
            .real_uploaded_delta
            .saturating_add(new.real_uploaded_delta);
        self.real_downloaded_delta = self
            .real_downloaded_delta
            .saturating_add(new.real_downloaded_delta);
    }
}

impl Flushable<PeerUpdate> for Mutex<Queue<Index, PeerUpdate>> {
    async fn flush_to_database(&self, db: &PgPool) -> u64 {
        let amount_of_updates = self.lock().records.len();
        let updates = self
            .lock()
            .records
            .drain(0..amount_of_updates)
            .collect::<Vec<(Index, PeerUpdate)>>();
        if updates.is_empty() {
            return 0;
        }

        let mut user_ids: Vec<i32> = Vec::with_capacity(updates.len());
        let mut torrent_ids: Vec<i32> = Vec::with_capacity(updates.len());
        let mut peer_ids: Vec<Vec<u8>> = Vec::with_capacity(updates.len());
        let mut ips: Vec<IpNetwork> = Vec::with_capacity(updates.len());
        let mut ports: Vec<i32> = Vec::with_capacity(updates.len());
        let mut agents: Vec<String> = Vec::with_capacity(updates.len());
        let mut uploadeds: Vec<i64> = Vec::with_capacity(updates.len());
        let mut downloadeds: Vec<i64> = Vec::with_capacity(updates.len());
        let mut lefts: Vec<i64> = Vec::with_capacity(updates.len());
        let mut actives: Vec<bool> = Vec::with_capacity(updates.len());
        let mut seeders: Vec<bool> = Vec::with_capacity(updates.len());
        let mut created_ats: Vec<DateTime<Utc>> = Vec::with_capacity(updates.len());
        let mut updated_ats: Vec<DateTime<Utc>> = Vec::with_capacity(updates.len());
        // torrent_activity fields
        let mut completed_ats: Vec<Option<DateTime<Utc>>> = Vec::with_capacity(updates.len());
        let mut uploaded_deltas: Vec<i64> = Vec::with_capacity(updates.len());
        let mut downloaded_deltas: Vec<i64> = Vec::with_capacity(updates.len());
        let mut real_uploaded_deltas: Vec<i64> = Vec::with_capacity(updates.len());
        let mut real_downloaded_deltas: Vec<i64> = Vec::with_capacity(updates.len());

        for (index, update) in updates {
            user_ids.push(index.user_id as i32);
            torrent_ids.push(index.torrent_id as i32);
            peer_ids.push(index.peer_id.to_vec());
            ips.push(IpNetwork::from(update.ip));
            ports.push(update.port as i32);
            agents.push(update.agent.clone());
            uploadeds.push(update.uploaded as i64);
            downloadeds.push(update.downloaded as i64);
            lefts.push(update.left as i64);
            actives.push(update.is_active);
            seeders.push(update.is_seeder);
            created_ats.push(update.created_at);
            updated_ats.push(update.updated_at);
            // torrent_activities fields
            completed_ats.push(update.completed_at);
            uploaded_deltas.push(update.uploaded_delta as i64);
            downloaded_deltas.push(update.downloaded_delta as i64);
            real_uploaded_deltas.push(update.real_uploaded_delta as i64);
            real_downloaded_deltas.push(update.real_downloaded_delta as i64);
        }

        let result = sqlx::query!(
            r#"
                INSERT INTO peers (
                    peer_id,
                    ip,
                    port,
                    agent,
                    uploaded,
                    downloaded,
                    "left",
                    active,
                    seeder,
                    created_at,
                    updated_at,
                    torrent_id,
                    user_id
                )
                SELECT
                    t.peer_id,
                    t.ip,
                    t.port,
                    t.agent,
                    t.uploaded,
                    t.downloaded,
                    t."left",
                    t.active,
                    t.seeder,
                    -- stored as timestamp without time zone in DB
                    (t.created_at AT TIME ZONE 'UTC')::timestamp,
                    (t.updated_at AT TIME ZONE 'UTC')::timestamp,
                    t.torrent_id,
                    t.user_id
                FROM (
                    SELECT * FROM unnest(
                        $1::bytea[],
                        $2::inet[],
                        $3::int[],
                        $4::varchar[],
                        $5::bigint[],
                        $6::bigint[],
                        $7::bigint[],
                        $8::boolean[],
                        $9::boolean[],
                        $10::timestamptz[],
                        $11::timestamptz[],
                        $12::int[],
                        $13::int[]
                    ) AS t(
                        peer_id,
                        ip,
                        port,
                        agent,
                        uploaded,
                        downloaded,
                        "left",
                        active,
                        seeder,
                        created_at,
                        updated_at,
                        torrent_id,
                        user_id
                    )
                ) AS t
                ON CONFLICT (user_id, torrent_id, peer_id) DO UPDATE SET
                    ip = EXCLUDED.ip,
                    port = EXCLUDED.port,
                    agent = EXCLUDED.agent,
                    uploaded = EXCLUDED.uploaded,
                    downloaded = EXCLUDED.downloaded,
                    "left" = EXCLUDED."left",
                    active = EXCLUDED.active,
                    seeder = EXCLUDED.seeder,
                    updated_at = EXCLUDED.updated_at
            "#,
            &peer_ids,
            &ips,
            &ports,
            &agents,
            &uploadeds,
            &downloadeds,
            &lefts,
            &actives,
            &seeders,
            &created_ats,
            &updated_ats,
            &torrent_ids,
            &user_ids
        )
        .execute(db)
        .await
        .map_err(|e| Error::DatabseError(e.to_string()));

        let peers_rows_affected = match result {
            Ok(query_result) => {
                log::info!("Inserted {amount_of_updates} peer updates");
                query_result.rows_affected()
            }
            Err(error) => {
                // TODO: reinsert the updates that failed and retry
                log::error!("Failed to insert peer updates: {error}");
                0
            }
        };

        let torrent_activities_result = sqlx::query!(
            r#"
                INSERT INTO torrent_activities (
                    torrent_id,
                    user_id,
                    completed_at,
                    first_seen_seeding_at,
                    last_seen_seeding_at,
                    total_seed_time,
                    uploaded,
                    real_uploaded,
                    downloaded,
                    real_downloaded
                )
                SELECT
                    agg.torrent_id,
                    agg.user_id,
                    agg.completed_at,
                    agg.first_seen_seeding_at,
                    agg.last_seen_seeding_at,
                    0,
                    agg.uploaded_delta,
                    agg.real_uploaded_delta,
                    agg.downloaded_delta,
                    agg.real_downloaded_delta
                FROM (
                    SELECT
                        t.torrent_id,
                        t.user_id,
                        MIN(t.completed_at) AS completed_at,
                        MIN(CASE WHEN t.seeder THEN t.updated_at END) AS first_seen_seeding_at,
                        MAX(CASE WHEN t.seeder THEN t.updated_at END) AS last_seen_seeding_at,
                        SUM(t.uploaded_delta) AS uploaded_delta,
                        SUM(t.real_uploaded_delta) AS real_uploaded_delta,
                        SUM(t.downloaded_delta) AS downloaded_delta,
                        SUM(t.real_downloaded_delta) AS real_downloaded_delta
                    FROM unnest(
                        $1::int[],
                        $2::int[],
                        $3::timestamptz[],
                        $4::bigint[],
                        $5::bigint[],
                        $6::bigint[],
                        $7::bigint[],
                        $8::boolean[],
                        $9::timestamptz[]
                    ) AS t(
                        torrent_id,
                        user_id,
                        completed_at,
                        uploaded_delta,
                        downloaded_delta,
                        real_uploaded_delta,
                        real_downloaded_delta,
                        seeder,
                        updated_at
                    )
                    GROUP BY t.torrent_id, t.user_id
                ) AS agg
                ON CONFLICT (torrent_id, user_id) DO UPDATE SET
                    completed_at = COALESCE(torrent_activities.completed_at, EXCLUDED.completed_at),
                    first_seen_seeding_at = COALESCE(torrent_activities.first_seen_seeding_at, EXCLUDED.first_seen_seeding_at),
                    last_seen_seeding_at = GREATEST(torrent_activities.last_seen_seeding_at, EXCLUDED.last_seen_seeding_at),
                    total_seed_time = torrent_activities.total_seed_time,
                    uploaded = torrent_activities.uploaded + COALESCE(EXCLUDED.uploaded, 0),
                    real_uploaded = torrent_activities.real_uploaded + COALESCE(EXCLUDED.real_uploaded, 0),
                    downloaded = torrent_activities.downloaded + COALESCE(EXCLUDED.downloaded, 0),
                    real_downloaded = torrent_activities.real_downloaded + COALESCE(EXCLUDED.real_downloaded, 0)
            "#,
            &torrent_ids,
            &user_ids,
            &completed_ats as &[Option<DateTime<Utc>>],
            &uploaded_deltas,
            &downloaded_deltas,
            &real_uploaded_deltas,
            &real_downloaded_deltas,
            &seeders,
            &updated_ats
        )
        .execute(db)
        .await
        .map_err(|e| Error::DatabseError(e.to_string()));

        let torrent_activities_rows_affected = match torrent_activities_result {
            Ok(query_result) => query_result.rows_affected(),
            Err(error) => {
                log::error!("Failed to update torrent_activities: {error}");
                0
            }
        };

        peers_rows_affected + torrent_activities_rows_affected
    }
}
