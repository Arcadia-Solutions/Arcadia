use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        user::UserPermission,
        user_badge::{UserBadgeManualAward, UserEarnedBadge},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Award user badge",
    tag = "User Badge",
    path = "/api/user-badges/award",
    security(("http" = ["Bearer"])),
    responses(
        (status = 201, description = "Successfully awarded the badge", body=UserEarnedBadge),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    award: Json<UserBadgeManualAward>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::AwardUserBadge, req.path())
        .await?;

    let earned = arc
        .pool
        .award_user_badge(
            award.user_id,
            award.badge_id,
            Some(user.sub),
            award.note.as_deref(),
        )
        .await?;

    Ok(HttpResponse::Created().json(earned))
}
