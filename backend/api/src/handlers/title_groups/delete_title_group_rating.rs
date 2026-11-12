use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Delete title group rating",
    tag = "Title Group",
    path = "/api/title-groups/ratings/{id}",
    params(
        ("id" = i64, Path, description = "Rating id")
    ),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the rating"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i64>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // authors deliberately can't delete their own rating: once rated, a title group stays rated
    // by them, they can only edit the rating
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::DeleteTitleGroupRating,
            req.path(),
        )
        .await?;

    arc.pool
        .delete_title_group_rating(path.into_inner())
        .await?;

    Ok(HttpResponse::Ok().finish())
}
