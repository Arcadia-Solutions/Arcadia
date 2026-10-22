use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use utoipa::ToSchema;

use super::user::UserLite;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TorrentReport {
    pub id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub reported_at: DateTime<Local>,
    #[schema(value_type = UserLite)]
    pub reported_by: Json<UserLite>,
    pub reported_torrent_id: i32,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedTorrentReport {
    pub reported_torrent_id: i32,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct DeleteTorrentReportQuery {
    pub torrent_report_id: i64,
}
