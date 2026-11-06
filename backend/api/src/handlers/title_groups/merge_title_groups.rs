use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data, web::Query};
use arcadia_common::error::{Error, Result};
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct MergeTitleGroupsQuery {
    pub source_title_group_id: i32,
    pub target_title_group_id: i32,
}

#[utoipa::path(
    post,
    operation_id = "Merge title groups",
    tag = "Title Group",
    path = "/api/title-groups/merge",
    security(
        ("http" = ["Bearer"])
    ),
    params(MergeTitleGroupsQuery),
    responses(
        (status = 200, description = "Successfully merged the title groups"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<MergeTitleGroupsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    if query.source_title_group_id == query.target_title_group_id {
        return Err(Error::CannotMergeTitleGroupIntoItself);
    }

    let has_permission = arc
        .pool
        .require_permission(user.sub, &UserPermission::MergeTitleGroup, req.path())
        .await;

    let source_title_group = arc
        .pool
        .find_title_group(query.source_title_group_id)
        .await?;

    if has_permission.is_err() {
        let is_creator = source_title_group.created_by_id == user.sub;
        if !is_creator {
            return Err(Error::InsufficientPermissions(
                "MergeTitleGroup".to_string(),
            ));
        }
    }

    let target_title_group = arc
        .pool
        .find_title_group(query.target_title_group_id)
        .await?;

    if source_title_group.content_type != target_title_group.content_type {
        return Err(Error::CannotMergeTitleGroupsWithDifferentContentTypes);
    }

    arc.pool
        .merge_title_groups(query.source_title_group_id, query.target_title_group_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
