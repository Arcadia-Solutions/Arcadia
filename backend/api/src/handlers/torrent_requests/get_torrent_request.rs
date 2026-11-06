use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::torrent_request::TorrentRequestAndAssociatedData, redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetTorrentRequestQuery {
    id: i64,
}

#[utoipa::path(
    get,
    operation_id = "Get torrent request",
    tag = "Torrent Request",
    path = "/api/torrent-requests",
    params(GetTorrentRequestQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the torrent request with associated data", body=TorrentRequestAndAssociatedData),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<GetTorrentRequestQuery>,
    user: Authdata,
) -> Result<HttpResponse> {
    let torrent_request = arc
        .pool
        .find_torrent_request_hierarchy(query.id, user.sub)
        .await?;

    // Mark any torrent request comment notifications as read for this user
    let _ = arc
        .pool
        .mark_notification_torrent_request_comment_as_read(query.id, user.sub)
        .await;

    Ok(HttpResponse::Ok().json(torrent_request))
}
