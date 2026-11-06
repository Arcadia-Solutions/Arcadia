use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        unauthorized_access::{SearchUnauthorizedAccessQuery, UnauthorizedAccess},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search unauthorized access logs",
    tag = "Unauthorized Access",
    path = "/api/unauthorized-access",
    params(SearchUnauthorizedAccessQuery),
    responses(
        (status = 200, description = "Paginated list of unauthorized access logs", body = PaginatedResults<UnauthorizedAccess>),
        (status = 403, description = "Forbidden"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    query: Query<SearchUnauthorizedAccessQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::SearchUnauthorizedAccess,
            req.path(),
        )
        .await?;

    let results = arc
        .pool
        .find_unauthorized_accesses(query.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
