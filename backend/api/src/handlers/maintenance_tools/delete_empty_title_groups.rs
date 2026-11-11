use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteEmptyTitleGroupsRequest {
    /// Only delete title groups created at or after this date
    #[schema(value_type = Option<String>, format = DateTime)]
    pub created_after: Option<DateTime<Utc>>,
    /// Only delete title groups created strictly before this date
    #[schema(value_type = Option<String>, format = DateTime)]
    pub created_before: Option<DateTime<Utc>>,
    /// Only delete these title groups among the empty ones, all of them if not given
    pub title_group_ids: Option<Vec<i32>>,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteEmptyTitleGroupsResponse {
    pub deleted_title_groups_amount: u64,
}

#[utoipa::path(
    post,
    operation_id = "Delete empty title groups",
    tag = "Maintenance Tools",
    path = "/api/maintenance-tools/delete-empty-title-groups",
    security(
        ("http" = ["Bearer"])
    ),
    request_body = DeleteEmptyTitleGroupsRequest,
    responses(
        (status = 200, description = "Successfully deleted the title groups without any undeleted torrent nor torrent request", body = DeleteEmptyTitleGroupsResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<DeleteEmptyTitleGroupsRequest>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::UseMaintenanceTools, req.path())
        .await?;

    let deleted_title_groups_amount = arc
        .pool
        .delete_empty_title_groups(
            form.created_after,
            form.created_before,
            form.title_group_ids.as_deref(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(DeleteEmptyTitleGroupsResponse {
        deleted_title_groups_amount,
    }))
}
