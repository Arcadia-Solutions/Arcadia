use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Delete torrent request comment",
    tag = "Torrent Request",
    path = "/api/torrent-requests/comment/{id}",
    params(
        ("id" = i64, Path, description = "Comment id")
    ),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the comment"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i64>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::DeleteTorrentRequestComment,
            req.path(),
        )
        .await?;

    arc.pool
        .delete_torrent_request_comment(path.into_inner())
        .await?;

    Ok(HttpResponse::Ok().finish())
}
