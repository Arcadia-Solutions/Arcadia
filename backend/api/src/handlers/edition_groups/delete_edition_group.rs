use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use chrono::Utc;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct DeleteEditionGroupQuery {
    pub edition_group_id: i32,
}

#[utoipa::path(
    delete,
    operation_id = "Delete edition group",
    tag = "Edition Group",
    path = "/api/edition-groups",
    security(
        ("http" = ["Bearer"])
    ),
    params(DeleteEditionGroupQuery),
    responses(
        (status = 200, description = "Successfully deleted the edition group"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<DeleteEditionGroupQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let has_permission = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::DeleteEditionGroup)
        .await?;

    if !has_permission {
        let edition_group = arc.pool.find_edition_group(query.edition_group_id).await?;

        if edition_group.created_by_id != user.sub {
            return Err(Error::InsufficientPermissions(format!(
                "{:?}",
                UserPermission::DeleteEditionGroup
            )));
        }

        let hours_since_creation = (Utc::now() - edition_group.created_at).num_hours();
        if hours_since_creation >= 24 {
            return Err(Error::EditionGroupDeletionWindowExpired);
        }
    }

    arc.pool
        .delete_edition_group(query.edition_group_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
