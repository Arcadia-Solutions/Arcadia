use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use reqwest::Method;

use chrono::Local;

use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{torrent::TorrentToDelete, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    delete,
    operation_id = "Delete torrent",
    tag = "Torrent",
    path = "/api/torrents",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Torrent deleted"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<TorrentToDelete>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let has_permission = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::DeleteTorrent)
        .await?;

    if !has_permission {
        let torrent = arc.pool.find_torrent(form.id).await?;

        if torrent.created_by_id != user.sub {
            arc.pool
                .require_permission(user.sub, &UserPermission::DeleteTorrent, req.path())
                .await?;
        }

        let hours_since_upload = (Local::now() - torrent.created_at).num_hours();
        if hours_since_upload >= 24 {
            return Err(Error::TorrentDeletionWindowExpired);
        }
    }

    arc.pool
        .remove_torrent(&form, user.sub, &arc.notification_sender)
        .await?;

    arc.pool
        .notify_tracker(
            Method::DELETE,
            &["torrents", &form.id.to_string()],
            None::<&()>,
            "mark torrent as deleted",
        )
        .await;

    Ok(HttpResponse::Ok().finish())
}
