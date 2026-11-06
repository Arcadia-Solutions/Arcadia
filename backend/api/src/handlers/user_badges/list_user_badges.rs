use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{user::UserPermission, user_badge::UserBadgeListItem},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "List user badges",
    tag = "User Badge",
    path = "/api/user-badges",
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "List of user badges (secret badges hidden if viewer lacks view_invisible_user_badges)", body=Vec<UserBadgeListItem>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let viewer_can_see_secret = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::ViewInvisibleUserBadges)
        .await?;

    let badges = arc.pool.find_all_user_badges(viewer_can_see_secret).await?;
    Ok(HttpResponse::Ok().json(badges))
}
