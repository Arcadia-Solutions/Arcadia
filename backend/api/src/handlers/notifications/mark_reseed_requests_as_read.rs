use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
use serde_json::json;

#[utoipa::path(
    post,
    operation_id = "Mark reseed request notifications as read",
    tag = "Notification",
    path = "/api/notifications/reseed-requests/{torrent_id}/read",
    params(("torrent_id" = i32, Path, description = "Torrent id")),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "The reseed request notification for this torrent was marked as read"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    torrent_id: Path<i32>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .mark_notifications_reseed_requests_as_read(user.sub, torrent_id.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
