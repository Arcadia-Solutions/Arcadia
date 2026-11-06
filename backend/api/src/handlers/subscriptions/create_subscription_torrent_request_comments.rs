use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct AddSubscriptionTorrentRequestCommentsQuery {
    pub torrent_request_id: i64,
}

#[utoipa::path(
    post,
    operation_id = "Create torrent request comments subscription",
    tag = "Subscription",
    path = "/api/subscriptions/torrent-request-comments",
    params (AddSubscriptionTorrentRequestCommentsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully subscribed to the item"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<AddSubscriptionTorrentRequestCommentsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .create_subscription_torrent_request_comments(query.torrent_request_id, user.sub)
        .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({"result": "success"})))
}
