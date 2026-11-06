use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        user::UserPermission,
        user_badge::{EditedUserBadgeCategory, UserBadgeCategory},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit user badge category",
    tag = "User Badge Category",
    path = "/api/user-badge-categories",
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Successfully edited the user badge category", body=UserBadgeCategory),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    edited: Json<EditedUserBadgeCategory>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditUserBadgeCategory, req.path())
        .await?;

    let updated = arc.pool.update_user_badge_category(&edited).await?;
    Ok(HttpResponse::Ok().json(updated))
}
