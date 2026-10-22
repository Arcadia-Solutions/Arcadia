use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::Display;
use utoipa::ToSchema;

use utoipa::IntoParams;

use super::{common::OrderByDirection, title_group::TitleGroupHierarchyLite, user::UserLite};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GetTorrentActivitiesQuery {
    pub page: u32,
    pub page_size: u32,
    pub include_unseeded_torrents: bool,
    pub order_by_column: TorrentActivityOrderByColumn,
    pub order_by_direction: OrderByDirection,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, Display)]
pub enum TorrentActivityOrderByColumn {
    #[serde(rename = "grabbed_at")]
    #[strum(serialize = "grabbed_at")]
    GrabbedAt,
    #[serde(rename = "total_seed_time")]
    #[strum(serialize = "total_seed_time")]
    TotalSeedTime,
    #[serde(rename = "uploaded")]
    #[strum(serialize = "uploaded")]
    Uploaded,
    #[serde(rename = "downloaded")]
    #[strum(serialize = "downloaded")]
    Downloaded,
    #[serde(rename = "torrent_size")]
    #[strum(serialize = "torrent_size")]
    TorrentSize,
    #[serde(rename = "torrent_seeders")]
    #[strum(serialize = "torrent_seeders")]
    TorrentSeeders,
    #[serde(rename = "bonus_points")]
    #[strum(serialize = "bonus_points")]
    BonusPoints,
    #[serde(rename = "bonus_points_per_day")]
    #[strum(serialize = "bonus_points_per_day")]
    BonusPointsPerDay,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, Display)]
pub enum TorrentActivityOfTorrentOrderByColumn {
    #[serde(rename = "username")]
    #[strum(serialize = "username")]
    Username,
    #[serde(rename = "seeder")]
    #[strum(serialize = "seeder")]
    Seeder,
    #[serde(rename = "uploaded")]
    #[strum(serialize = "uploaded")]
    Uploaded,
    #[serde(rename = "real_uploaded")]
    #[strum(serialize = "real_uploaded")]
    RealUploaded,
    #[serde(rename = "downloaded")]
    #[strum(serialize = "downloaded")]
    Downloaded,
    #[serde(rename = "real_downloaded")]
    #[strum(serialize = "real_downloaded")]
    RealDownloaded,
    #[serde(rename = "total_seed_time")]
    #[strum(serialize = "total_seed_time")]
    TotalSeedTime,
    #[serde(rename = "bonus_points")]
    #[strum(serialize = "bonus_points")]
    BonusPoints,
    #[serde(rename = "grabbed_at")]
    #[strum(serialize = "grabbed_at")]
    GrabbedAt,
    #[serde(rename = "completed_at")]
    #[strum(serialize = "completed_at")]
    CompletedAt,
    #[serde(rename = "first_seen_seeding_at")]
    #[strum(serialize = "first_seen_seeding_at")]
    FirstSeenSeedingAt,
    #[serde(rename = "last_seen_seeding_at")]
    #[strum(serialize = "last_seen_seeding_at")]
    LastSeenSeedingAt,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TorrentActivity {
    pub id: i64,
    pub torrent_id: i32,
    pub user_id: i32,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub grabbed_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub completed_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub first_seen_seeding_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_seen_seeding_at: Option<DateTime<Utc>>,
    pub total_seed_time: i64,
    pub bonus_points: i64,
    pub uploaded: i64,
    pub real_uploaded: i64,
    pub downloaded: i64,
    pub real_downloaded: i64,
    pub seeder: bool,
    pub bonus_points_per_day: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TorrentActivityAndTitleGroup {
    pub title_group: TitleGroupHierarchyLite,
    pub torrent_activity: TorrentActivity,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TorrentActivitiesOverview {
    pub bonus_points_per_day: i64,
    pub bonus_points_formula: String,
    pub bonus_points_update_interval_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TorrentActivityAndUser {
    pub user: UserLite,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub grabbed_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub completed_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub first_seen_seeding_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_seen_seeding_at: Option<DateTime<Utc>>,
    pub total_seed_time: i64,
    pub bonus_points: i64,
    pub uploaded: i64,
    pub real_uploaded: i64,
    pub downloaded: i64,
    pub real_downloaded: i64,
    pub seeder: bool,
}
