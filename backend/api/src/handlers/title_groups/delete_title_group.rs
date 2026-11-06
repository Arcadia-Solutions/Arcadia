use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data, web::Query};
use arcadia_common::error::Result;
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct DeleteTitleGroupQuery {
    pub title_group_id: i32,
}

#[utoipa::path(
    delete,
    operation_id = "Delete title group",
    tag = "Title Group",
    path = "/api/title-groups",
    security(
        ("http" = ["Bearer"])
    ),
    params(DeleteTitleGroupQuery),
    responses(
        (status = 200, description = "Successfully deleted the title group"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<DeleteTitleGroupQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteTitleGroup, req.path())
        .await?;

    arc.pool.delete_title_group(query.title_group_id).await?;

    Ok(HttpResponse::Ok().finish())
}
