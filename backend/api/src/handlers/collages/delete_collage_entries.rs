use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data, web::Query};
use arcadia_common::error::Result;
use arcadia_storage::models::collage::DeleteCollageEntriesQuery;
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
    delete,
    operation_id = "Delete collage entry",
    tag = "Collage",
    path = "/api/collages/entries",
    security(
        ("http" = ["Bearer"])
    ),
    params(DeleteCollageEntriesQuery),
    responses(
        (status = 200, description = "Successfully deleted the collage entry"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<DeleteCollageEntriesQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteCollageEntry, req.path())
        .await?;

    arc.pool
        .delete_collage_entry(query.collage_id, query.title_group_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
