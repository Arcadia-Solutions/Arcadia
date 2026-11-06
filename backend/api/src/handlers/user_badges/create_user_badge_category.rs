use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        user::UserPermission,
        user_badge::{UserBadgeCategory, UserCreatedUserBadgeCategory},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create user badge category",
    tag = "User Badge Category",
    path = "/api/user-badge-categories",
    security(("http" = ["Bearer"])),
    responses(
        (status = 201, description = "Successfully created the user badge category", body=UserBadgeCategory),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    category: Json<UserCreatedUserBadgeCategory>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::CreateUserBadgeCategory,
            req.path(),
        )
        .await?;

    let created = arc
        .pool
        .create_user_badge_category(&category, user.sub)
        .await?;
    Ok(HttpResponse::Created().json(created))
}
