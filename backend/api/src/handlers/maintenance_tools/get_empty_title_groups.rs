use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{title_group::TitleGroupHierarchyLite, user::UserPermission},
    redis::RedisPoolInterface,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct EmptyTitleGroupsQuery {
    /// Only title groups created at or after this date
    #[param(value_type = Option<String>, format = DateTime)]
    pub created_after: Option<DateTime<Utc>>,
    /// Only title groups created strictly before this date
    #[param(value_type = Option<String>, format = DateTime)]
    pub created_before: Option<DateTime<Utc>>,
    /// Only return the amount, `title_groups` is then empty
    #[serde(default)]
    pub count_only: bool,
}

#[derive(Serialize, ToSchema)]
pub struct EmptyTitleGroupsResponse {
    pub amount: u64,
    pub title_groups: Vec<TitleGroupHierarchyLite>,
}

#[utoipa::path(
    get,
    operation_id = "Get empty title groups",
    tag = "Maintenance Tools",
    path = "/api/maintenance-tools/empty-title-groups",
    security(
        ("http" = ["Bearer"])
    ),
    params(EmptyTitleGroupsQuery),
    responses(
        (status = 200, description = "Title groups without any undeleted torrent nor torrent request", body = EmptyTitleGroupsResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<EmptyTitleGroupsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::UseMaintenanceTools, req.path())
        .await?;

    let response = if query.count_only {
        EmptyTitleGroupsResponse {
            amount: arc
                .pool
                .count_empty_title_groups(query.created_after, query.created_before)
                .await?,
            title_groups: vec![],
        }
    } else {
        let title_groups = arc
            .pool
            .find_empty_title_groups(query.created_after, query.created_before)
            .await?;
        EmptyTitleGroupsResponse {
            amount: title_groups.len() as u64,
            title_groups,
        }
    };

    Ok(HttpResponse::Ok().json(response))
}
