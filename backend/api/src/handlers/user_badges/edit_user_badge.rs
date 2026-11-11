use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        user::UserPermission,
        user_badge::{EditedUserBadge, UserBadge},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit user badge",
    tag = "User Badge",
    path = "/api/user-badges",
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Successfully edited the user badge", body=UserBadge),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    edited: Json<EditedUserBadge>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditUserBadge, req.path())
        .await?;

    arc.validate_image_urls([&edited.image_url])?;

    let updated = arc.pool.update_user_badge(&edited).await?;
    Ok(HttpResponse::Ok().json(updated))
}
