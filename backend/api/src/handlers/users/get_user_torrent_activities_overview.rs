use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_periodic_tasks::config::formula_to_sql;
use arcadia_storage::{
    models::torrent_activity::TorrentActivitiesOverview, redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema)]
pub enum SeedersPerTorrent {
    #[serde(rename = "current")]
    Current,
    #[serde(rename = "1")]
    Fixed1,
    #[serde(rename = "2")]
    Fixed2,
    #[serde(rename = "3")]
    Fixed3,
    #[serde(rename = "4")]
    Fixed4,
    #[serde(rename = "5")]
    Fixed5,
    #[serde(rename = "10")]
    Fixed10,
    #[serde(rename = "15")]
    Fixed15,
    #[serde(rename = "20")]
    Fixed20,
    #[serde(rename = "25")]
    Fixed25,
    #[serde(rename = "50")]
    Fixed50,
    #[serde(rename = "100")]
    Fixed100,
    #[serde(rename = "10_percent")]
    Percent10,
    #[serde(rename = "25_percent")]
    Percent25,
    #[serde(rename = "50_percent")]
    Percent50,
    #[serde(rename = "75_percent")]
    Percent75,
    #[serde(rename = "100_percent")]
    Percent100,
    #[serde(rename = "150_percent")]
    Percent150,
    #[serde(rename = "200_percent")]
    Percent200,
    #[serde(rename = "300_percent")]
    Percent300,
    #[serde(rename = "500_percent")]
    Percent500,
}

impl SeedersPerTorrent {
    pub fn to_seeders_sql(&self) -> &'static str {
        match self {
            SeedersPerTorrent::Current | SeedersPerTorrent::Percent100 => "t.seeders",
            SeedersPerTorrent::Fixed1 => "1",
            SeedersPerTorrent::Fixed2 => "2",
            SeedersPerTorrent::Fixed3 => "3",
            SeedersPerTorrent::Fixed4 => "4",
            SeedersPerTorrent::Fixed5 => "5",
            SeedersPerTorrent::Fixed10 => "10",
            SeedersPerTorrent::Fixed15 => "15",
            SeedersPerTorrent::Fixed20 => "20",
            SeedersPerTorrent::Fixed25 => "25",
            SeedersPerTorrent::Fixed50 => "50",
            SeedersPerTorrent::Fixed100 => "100",
            SeedersPerTorrent::Percent10 => "GREATEST(t.seeders / 10, 1)",
            SeedersPerTorrent::Percent25 => "GREATEST(t.seeders / 4, 1)",
            SeedersPerTorrent::Percent50 => "GREATEST(t.seeders / 2, 1)",
            SeedersPerTorrent::Percent75 => "GREATEST(t.seeders * 3 / 4, 1)",
            SeedersPerTorrent::Percent150 => "t.seeders * 3 / 2",
            SeedersPerTorrent::Percent200 => "t.seeders * 2",
            SeedersPerTorrent::Percent300 => "t.seeders * 3",
            SeedersPerTorrent::Percent500 => "t.seeders * 5",
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetTorrentActivitiesOverviewQuery {
    pub hours_seeding_per_day: u8,
    pub seeders_per_torrent: SeedersPerTorrent,
}

#[utoipa::path(
    get,
    operation_id = "Get torrent activities overview",
    tag = "User",
    path = "/api/users/torrent-activities/overview",
    params(GetTorrentActivitiesOverviewQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the torrent activities overview", body = TorrentActivitiesOverview),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    query: Query<GetTorrentActivitiesOverviewQuery>,
) -> Result<HttpResponse> {
    let seeders_sql = query.seeders_per_torrent.to_seeders_sql();
    let formula_sql = formula_to_sql(&arc.periodic_tasks.bonus_points_formula, seeders_sql)
        .map_err(|e| arcadia_common::error::Error::InvalidBonusPointsFormula(e.to_string()))?;

    let bonus_per_tick = arc
        .pool
        .get_torrent_activities_overview(user.sub, &formula_sql)
        .await?;

    let task_interval = arc.periodic_tasks.seedtime_and_bonus_points_update_seconds;
    let ticks_per_day = (query.hours_seeding_per_day as u64 * 3600) / task_interval;
    let bonus_points_per_day = bonus_per_tick * ticks_per_day as i64;

    let overview = TorrentActivitiesOverview {
        bonus_points_per_day,
        bonus_points_formula: arc.periodic_tasks.bonus_points_formula.clone(),
        bonus_points_update_interval_seconds: task_interval,
    };

    Ok(HttpResponse::Ok().json(overview))
}
