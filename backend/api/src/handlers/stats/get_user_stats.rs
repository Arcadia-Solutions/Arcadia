use crate::Arcadia;
use actix_web::{HttpRequest, HttpResponse, web::Data, web::Query};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        user::UserPermission,
        user_stats::{UserStatsQuery, UserStatsResponse},
    },
    redis::RedisPoolInterface,
};

use crate::middlewares::auth_middleware::Authdata;

#[utoipa::path(
    get,
    operation_id = "Get user stats",
    tag = "Stats",
    path = "/api/stats/users",
    params(UserStatsQuery),
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "User stats", body = UserStatsResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<UserStatsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ViewStatsDetails, req.path())
        .await?;

    let response = arc.pool.get_users_stats(&query).await?;

    Ok(HttpResponse::Ok().json(response))
}
