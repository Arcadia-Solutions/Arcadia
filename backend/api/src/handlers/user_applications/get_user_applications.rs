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
        user_application::{UserApplicationHierarchy, UserApplicationStatus},
    },
    redis::RedisPoolInterface,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct GetUserApplicationsQuery {
    pub page_size: Option<i64>,
    pub page: Option<i64>,
    pub status: Option<UserApplicationStatus>,
}

#[utoipa::path(
    get,
    operation_id = "Get user applications",
    tag = "User Application",
    path = "/api/user-applications",
    params(
        ("page_size" = Option<i64>, Query, description = "Maximum number of applications to return (default: 50)"),
        ("page" = Option<i64>, Query, description = "Page (default: 1)"),
        ("status" = Option<String>, Query, description = "Filter by application status: 'pending', 'accepted', or 'rejected'")
    ),
    responses(
        (status = 200, description = "Successfully retrieved user applications", body = PaginatedResults<UserApplicationHierarchy>),
        (status = 400, description = "Bad Request - Invalid status parameter"),
        (status = 403, description = "Forbidden - Only staff members can view user applications")
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    query: Query<GetUserApplicationsQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::GetUserApplication, req.path())
        .await?;

    let applications = arc
        .pool
        .find_user_applications(
            query.page_size.unwrap_or(50),
            query.page.unwrap_or(1),
            query.status.clone(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(applications))
}
