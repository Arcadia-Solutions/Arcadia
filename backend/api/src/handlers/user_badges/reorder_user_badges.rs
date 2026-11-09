use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{user::UserPermission, user_badge::ReorderUserBadges},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Reorder user badges",
    tag = "User Badge",
    path = "/api/user-badges/reorder",
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Successfully reordered the user badges"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    reorder: Json<ReorderUserBadges>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditUserBadge, req.path())
        .await?;

    arc.pool.reorder_user_badges(&reorder).await?;

    Ok(HttpResponse::Ok().finish())
}
