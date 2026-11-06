use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GetTorrentTitleGroupQuery {
    torrent_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TorrentTitleGroupId {
    title_group_id: i32,
}

#[utoipa::path(
    get,
    operation_id = "Get torrent title group id",
    tag = "Torrent",
    path = "/api/torrents/title-group-id",
    params(GetTorrentTitleGroupQuery),
    responses(
        (status = 200, description = "Title group ID for the torrent", body=TorrentTitleGroupId),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetTorrentTitleGroupQuery>,
    arc: Data<Arcadia<R>>,
    _user: Authdata,
) -> Result<HttpResponse> {
    let title_group_id = arc
        .pool
        .get_torrent_title_group_id(query.torrent_id)
        .await?;

    Ok(HttpResponse::Ok().json(TorrentTitleGroupId { title_group_id }))
}
