use crate::Arcadia;
use actix_web::{HttpRequest, HttpResponse, web::Data, web::Query};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum_stats::{ForumStatsQuery, ForumStatsResponse},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

use crate::middlewares::auth_middleware::Authdata;

#[utoipa::path(
    get,
    operation_id = "Get forum stats",
    tag = "Stats",
    path = "/api/stats/forum",
    params(ForumStatsQuery),
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Forum stats", body = ForumStatsResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<ForumStatsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ViewStatsDetails, req.path())
        .await?;

    let response = arc.pool.get_forum_stats(&query).await?;

    Ok(HttpResponse::Ok().json(response))
}
