use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Delete title group review",
    tag = "Title Group",
    path = "/api/title-groups/reviews/{id}",
    params(
        ("id" = i64, Path, description = "Review id")
    ),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the review"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i64>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // authors deliberately can't delete their own review: once rated, a title group stays rated
    // by them, they can only edit the rating
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::DeleteTitleGroupReview,
            req.path(),
        )
        .await?;

    arc.pool
        .delete_title_group_review(path.into_inner())
        .await?;

    Ok(HttpResponse::Ok().finish())
}
