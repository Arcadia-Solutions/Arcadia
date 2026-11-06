use crate::{
    Arcadia, handlers::users::get_user_torrent_activities_overview::SeedersPerTorrent,
    middlewares::auth_middleware::Authdata,
};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_periodic_tasks::config::formula_to_sql;
use arcadia_storage::{
    models::{
        common::{OrderByDirection, PaginatedResults},
        torrent_activity::{
            GetTorrentActivitiesQuery, TorrentActivityAndTitleGroup, TorrentActivityOrderByColumn,
        },
    },
    redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetUserTorrentActivitiesQuery {
    pub page: u32,
    pub page_size: u32,
    pub include_unseeded_torrents: bool,
    pub order_by_column: TorrentActivityOrderByColumn,
    pub order_by_direction: OrderByDirection,
    pub hours_seeding_per_day: u8,
    pub seeders_per_torrent: SeedersPerTorrent,
}

#[utoipa::path(
    get,
    operation_id = "Get user torrent activities",
    tag = "User",
    path = "/api/users/torrent-activities",
    params(GetUserTorrentActivitiesQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the user's torrent activities", body = PaginatedResults<TorrentActivityAndTitleGroup>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    query: Query<GetUserTorrentActivitiesQuery>,
) -> Result<HttpResponse> {
    let seeders_sql = query.seeders_per_torrent.to_seeders_sql();
    let formula_sql = formula_to_sql(&arc.periodic_tasks.bonus_points_formula, seeders_sql)
        .map_err(|e| arcadia_common::error::Error::InvalidBonusPointsFormula(e.to_string()))?;

    let task_interval = arc.periodic_tasks.seedtime_and_bonus_points_update_seconds;
    let ticks_per_day = (query.hours_seeding_per_day as i64 * 3600) / task_interval as i64;

    let activities_query = GetTorrentActivitiesQuery {
        page: query.page,
        page_size: query.page_size,
        include_unseeded_torrents: query.include_unseeded_torrents,
        order_by_column: query.order_by_column,
        order_by_direction: query.order_by_direction,
    };

    let results = arc
        .pool
        .get_torrent_activities(user.sub, &activities_query, &formula_sql, ticks_per_day)
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
