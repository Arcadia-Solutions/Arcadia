use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        site_highlight::{EditSiteHighlight, SiteHighlight},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit site highlight",
    tag = "Site Highlights",
    path = "/api/site-highlights/{id}",
    security(
        ("http" = ["Bearer"])
    ),
    params(("id" = i32, Path, description = "Site highlight id")),
    request_body = EditSiteHighlight,
    responses(
        (status = 200, description = "Successfully edited the site highlight", body=SiteHighlight),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i32>,
    payload: Json<EditSiteHighlight>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageSiteHighlights, req.path())
        .await?;

    let id = path.into_inner();
    let edited = arc.pool.edit_site_highlight(id, &payload).await?;
    Ok(HttpResponse::Ok().json(edited))
}
