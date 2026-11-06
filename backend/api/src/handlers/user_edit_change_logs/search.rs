use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        user::UserPermission,
        user_edit_change_log::{SearchUserEditChangeLogsQuery, UserEditChangeLogResult},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search user edit change logs",
    tag = "User Edit Change Logs",
    path = "/api/user-edit-change-logs",
    params(SearchUserEditChangeLogsQuery),
    responses(
        (status = 200, description = "Paginated list of user edit change logs", body = PaginatedResults<UserEditChangeLogResult>),
        (status = 403, description = "Forbidden"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    query: Query<SearchUserEditChangeLogsQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::SearchUserEditChangeLogs,
            req.path(),
        )
        .await?;

    let results = arc
        .pool
        .search_user_edit_change_logs(query.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
