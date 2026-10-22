use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use super::torrent_stats::StatsInterval;

#[derive(Debug, Deserialize, IntoParams)]
pub struct UserStatsQuery {
    #[param(value_type = String, format = "date")]
    pub from: chrono::NaiveDate,
    #[param(value_type = String, format = "date")]
    pub to: chrono::NaiveDate,
    pub interval: StatsInterval,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserStatsDataPoint {
    #[schema(value_type = String, format = DateTime)]
    pub period: NaiveDateTime,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TorrentClientStatsDataPoint {
    pub client: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CountryStatsDataPoint {
    /// ISO 3166-1 alpha-2
    pub country_code: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserStatsResponse {
    /// users that registered during the period
    pub new_users: i64,
    /// registrations per period
    pub data: Vec<UserStatsDataPoint>,
    /// torrent clients repartition
    pub torrent_clients: Vec<TorrentClientStatsDataPoint>,
    /// users repartition per country, from their registration ip
    pub countries: Vec<CountryStatsDataPoint>,
}
