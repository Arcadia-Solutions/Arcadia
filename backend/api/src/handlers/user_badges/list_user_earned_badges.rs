use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user_badge::UserEarnedBadges, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListUserEarnedBadgesQuery {
    user_id: i32,
}

#[utoipa::path(
    get,
    operation_id = "List user earned badges",
    tag = "User Badge",
    path = "/api/user-badges/earned",
    params(ListUserEarnedBadgesQuery),
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "The user and every badge earned by the user, most recently earned first", body=UserEarnedBadges),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<ListUserEarnedBadgesQuery>,
    arc: Data<Arcadia<R>>,
    _user: Authdata,
) -> Result<HttpResponse> {
    let user = arc.pool.find_user_lite(query.user_id).await?;
    let earned_badges = arc
        .pool
        .find_user_earned_badges(query.user_id, None)
        .await?;
    Ok(HttpResponse::Ok().json(UserEarnedBadges {
        user,
        earned_badges,
    }))
}
