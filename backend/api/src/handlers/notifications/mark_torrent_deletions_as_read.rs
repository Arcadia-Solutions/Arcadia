use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkTorrentDeletionsAsReadForm {
    pub torrent_ids: Vec<i32>,
}

#[utoipa::path(
    post,
    operation_id = "Mark torrent deletion notifications as read",
    tag = "Notification",
    path = "/api/notifications/torrent-deletions/read",
    request_body = MarkTorrentDeletionsAsReadForm,
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Selected torrent deletion notifications marked as read"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<MarkTorrentDeletionsAsReadForm>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    if !form.torrent_ids.is_empty() {
        arc.pool
            .mark_notifications_torrent_deletions_as_read(user.sub, &form.torrent_ids)
            .await?;
    }

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
