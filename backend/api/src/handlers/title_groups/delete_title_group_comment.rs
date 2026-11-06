use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Delete title group comment",
    tag = "Title Group",
    path = "/api/title-groups/comments/{id}",
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
            &UserPermission::DeleteTitleGroupComment,
            req.path(),
        )
        .await?;

    arc.pool
        .delete_title_group_comment(path.into_inner())
        .await?;

    Ok(HttpResponse::Ok().finish())
}
