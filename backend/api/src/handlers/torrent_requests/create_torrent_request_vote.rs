use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::torrent_request_vote::{TorrentRequestVote, UserCreatedTorrentRequestVote},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create torrent request vote",
    tag = "Torrent Request",
    path = "/api/torrent-requests/vote",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully voted on the torrent_request", body=TorrentRequestVote),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    torrent_request_vote: Json<UserCreatedTorrentRequestVote>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let vote_currencies = arc
        .settings
        .lock()
        .unwrap()
        .torrent_request_vote_currencies
        .clone();

    let vote = arc
        .pool
        .create_torrent_request_vote(&torrent_request_vote, user.sub, &vote_currencies)
        .await?;

    Ok(HttpResponse::Created().json(vote))
}
