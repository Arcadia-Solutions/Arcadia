use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Revoke user earned badge",
    tag = "User Badge",
    path = "/api/user-badges/award/{id}",
    params(("id" = i32, Path, description = "User earned badge ID")),
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Successfully revoked the badge from the user"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user_earned_badge_id: Path<i32>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::RevokeUserBadge, req.path())
        .await?;

    arc.pool
        .revoke_user_earned_badge(user_earned_badge_id.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}
