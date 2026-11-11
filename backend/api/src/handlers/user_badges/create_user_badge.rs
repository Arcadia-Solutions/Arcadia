use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        user::UserPermission,
        user_badge::{UserBadge, UserCreatedUserBadge},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create user badge",
    tag = "User Badge",
    path = "/api/user-badges",
    security(("http" = ["Bearer"])),
    responses(
        (status = 201, description = "Successfully created the user badge", body=UserBadge),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    badge: Json<UserCreatedUserBadge>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::CreateUserBadge, req.path())
        .await?;

    arc.validate_image_urls([&badge.image_url])?;

    let created = arc.pool.create_user_badge(&badge, user.sub).await?;
    Ok(HttpResponse::Created().json(created))
}
