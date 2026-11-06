use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        bonus_points_log::{BonusPointsLog, SearchBonusPointsLogsQuery},
        common::PaginatedResults,
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search bonus points logs",
    tag = "User",
    path = "/api/users/bonus-points-logs",
    params(SearchBonusPointsLogsQuery),
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Paginated bonus points logs of the searched user", body = PaginatedResults<BonusPointsLog>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    req: HttpRequest,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    query: Query<SearchBonusPointsLogsQuery>,
) -> Result<HttpResponse> {
    if query.user_id != user.sub {
        arc.pool
            .require_permission(
                user.sub,
                &UserPermission::SeeForeignBonusPointsLogs,
                req.path(),
            )
            .await?;
    }

    let results = arc.pool.search_bonus_points_logs(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
