use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{torrent_request::TorrentRequestUnfill, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Unfill torrent request",
    tag = "Torrent Request",
    path = "/api/torrent-requests/unfill",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully unfilled the torrent request"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<TorrentRequestUnfill>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let reason = form.reason.trim();
    if reason.is_empty() {
        return Err(Error::TorrentRequestUnfillReasonEmpty);
    }

    let has_permission = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::ManageUnfilledRequests)
        .await?;
    if !has_permission {
        let torrent_request = arc
            .pool
            .find_torrent_request(form.torrent_request_id)
            .await?;
        if torrent_request.created_by_id != user.sub {
            arc.pool
                .require_permission(
                    user.sub,
                    &UserPermission::ManageUnfilledRequests,
                    req.path(),
                )
                .await?;
        }
    }

    arc.pool
        .unfill_torrent_request(form.torrent_request_id, user.sub, reason)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
