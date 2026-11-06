use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Delete site highlight",
    tag = "Site Highlights",
    path = "/api/site-highlights/{id}",
    security(
        ("http" = ["Bearer"])
    ),
    params(("id" = i32, Path, description = "Site highlight id")),
    responses(
        (status = 200, description = "Successfully deleted the site highlight"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i32>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageSiteHighlights, req.path())
        .await?;

    let id = path.into_inner();
    arc.pool.delete_site_highlight(id).await?;
    Ok(HttpResponse::Ok().finish())
}
