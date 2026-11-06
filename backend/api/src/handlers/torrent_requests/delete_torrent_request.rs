use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        torrent_request::{TorrentRequestDeletionKind, TorrentRequestToDelete},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    delete,
    operation_id = "Delete torrent request",
    tag = "Torrent Request",
    path = "/api/torrent-requests",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the torrent request"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<TorrentRequestToDelete>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let has_permission = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::DeleteTorrentRequest)
        .await?;

    // whether the request is filled, and whether somebody else voted on it, is checked in the
    // transaction doing the deletion, so that a concurrent fill or vote cannot be lost
    let deletion_kind = if has_permission {
        TorrentRequestDeletionKind::WithPermission {
            refund_bounty: form.refund_bounty,
            message: form.message.as_deref(),
        }
    } else {
        let torrent_request = arc.pool.find_torrent_request(form.id).await?;
        if torrent_request.created_by_id != user.sub {
            arc.pool
                .require_permission(user.sub, &UserPermission::DeleteTorrentRequest, req.path())
                .await?;
        }

        TorrentRequestDeletionKind::Author
    };

    arc.pool
        .delete_torrent_request(form.id, user.sub, deletion_kind)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
