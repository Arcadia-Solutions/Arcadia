use actix_web::{
    web::{Data, Query},
    HttpRequest, HttpResponse,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{peer::PublicPeer, user::UserPermission},
    redis::RedisPoolInterface,
};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GetTorrentPeersQuery {
    torrent_id: i32,
}

#[utoipa::path(
    get,
    operation_id = "Get torrent peers",
    tag = "Torrent",
    path = "/api/torrents/peers",
    params(GetTorrentPeersQuery),
    responses(
        (status = 200, description = "List of peers for the torrent", body=Vec<PublicPeer>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetTorrentPeersQuery>,
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

    let peers = arc
        .pool
        .get_torrent_peers(query.torrent_id, user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(peers))
}
