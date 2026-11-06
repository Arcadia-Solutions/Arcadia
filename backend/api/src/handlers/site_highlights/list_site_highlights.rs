use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{site_highlight::SiteHighlight, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "List site highlights",
    tag = "Site Highlights",
    path = "/api/site-highlights",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully listed site highlights", body=Vec<SiteHighlight>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageSiteHighlights, req.path())
        .await?;

    let highlights = arc.pool.find_all_site_highlights().await?;
    Ok(HttpResponse::Ok().json(highlights))
}
