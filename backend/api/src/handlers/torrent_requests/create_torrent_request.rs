use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        torrent_request::{TorrentRequest, UserCreatedTorrentRequest},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create torrent request",
    tag = "Torrent Request",
    path = "/api/torrent-requests",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully created the torrent_request", body=TorrentRequest),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    mut torrent_request: Json<UserCreatedTorrentRequest>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::CreateTorrentRequest, req.path())
        .await?;

    let vote_currencies = arc
        .settings
        .lock()
        .unwrap()
        .torrent_request_vote_currencies
        .clone();

    let torrent_request = arc
        .pool
        .create_torrent_request(&mut torrent_request, user.sub, &vote_currencies)
        .await?;

    Ok(HttpResponse::Created().json(torrent_request))
}
