use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::torrent_request::TorrentRequestFill, redis::RedisPoolInterface};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TorrentRequestFillResponse {
    pub filler_is_uploader: bool,
}

#[utoipa::path(
    post,
    operation_id = "Fill torrent request",
    tag = "Torrent Request",
    path = "/api/torrent-requests/fill",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully filled the torrent request", body = TorrentRequestFillResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    torrent_request_fill: Json<TorrentRequestFill>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let filler_is_uploader = arc
        .pool
        .fill_torrent_request(
            torrent_request_fill.torrent_id,
            torrent_request_fill.torrent_request_id,
            user.sub,
        )
        .await?;

    Ok(HttpResponse::Ok().json(TorrentRequestFillResponse { filler_is_uploader }))
}
