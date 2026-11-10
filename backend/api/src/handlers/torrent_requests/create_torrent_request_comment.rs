use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::torrent_request_comment::{TorrentRequestComment, UserCreatedTorrentRequestComment},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create torrent request comment",
    tag = "Torrent Request",
    path = "/api/torrent-requests/comment",
    security(
      ("http" = ["Bearer"])
    ),
    request_body = UserCreatedTorrentRequestComment,
    responses(
        (status = 201, description = "Successfully commented on the torrent request", body = TorrentRequestComment),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    torrent_request_comment: Json<UserCreatedTorrentRequestComment>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let comment = arc
        .pool
        .create_torrent_request_comment(
            torrent_request_comment.torrent_request_id,
            user.sub,
            &torrent_request_comment.content,
        )
        .await?;

    Ok(HttpResponse::Created().json(comment))
}
