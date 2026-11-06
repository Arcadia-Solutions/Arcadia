use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data, web::Query};
use arcadia_common::error::Result;
use arcadia_storage::models::collage::DeleteCollageQuery;
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
    delete,
    operation_id = "Delete collage",
    tag = "Collage",
    path = "/api/collages",
    security(
        ("http" = ["Bearer"])
    ),
    params(DeleteCollageQuery),
    responses(
        (status = 200, description = "Successfully deleted the collage"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<DeleteCollageQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteCollage, req.path())
        .await?;

    arc.pool.delete_collage(query.collage_id).await?;

    Ok(HttpResponse::Ok().finish())
}
