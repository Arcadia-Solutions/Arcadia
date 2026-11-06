use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Delete user badge category",
    tag = "User Badge Category",
    path = "/api/user-badge-categories/{id}",
    params(("id" = i32, Path, description = "User badge category ID")),
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Successfully deleted the user badge category"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    category_id: Path<i32>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::DeleteUserBadgeCategory,
            req.path(),
        )
        .await?;

    arc.pool
        .delete_user_badge_category(category_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}
