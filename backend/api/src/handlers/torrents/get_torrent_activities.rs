use actix_web::{
    web::{Data, Query},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::{OrderByDirection, PaginatedResults},
        torrent_activity::{TorrentActivityAndUser, TorrentActivityOfTorrentOrderByColumn},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{middlewares::auth_middleware::Authdata, Arcadia};

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetTorrentActivitiesOfTorrentQuery {
    torrent_id: i32,
    page: u32,
    page_size: u32,
    order_by_column: TorrentActivityOfTorrentOrderByColumn,
    order_by_direction: OrderByDirection,
}

#[utoipa::path(
    get,
    operation_id = "Get torrent activities",
    tag = "Torrent",
    path = "/api/torrents/activities",
    params(GetTorrentActivitiesOfTorrentQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Activities of every user on the torrent", body = PaginatedResults<TorrentActivityAndUser>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetTorrentActivitiesOfTorrentQuery>,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::ViewTorrentPeersAndActivities,
            req.path(),
        )
        .await?;

    let activities = arc
        .pool
        .get_torrent_activities_of_torrent(
            query.torrent_id,
            query.page,
            query.page_size,
            query.order_by_column,
            query.order_by_direction,
        )
        .await?;

    Ok(HttpResponse::Ok().json(activities))
}
