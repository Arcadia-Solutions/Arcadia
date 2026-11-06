use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        site_highlight::{CreateSiteHighlight, SiteHighlight},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create site highlight",
    tag = "Site Highlights",
    path = "/api/site-highlights",
    security(
        ("http" = ["Bearer"])
    ),
    request_body = CreateSiteHighlight,
    responses(
        (status = 200, description = "Successfully created the site highlight", body=SiteHighlight),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    payload: Json<CreateSiteHighlight>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageSiteHighlights, req.path())
        .await?;

    if payload.alias.trim().is_empty() {
        return Err(Error::InvalidSiteHighlight(
            "alias must not be empty".to_string(),
        ));
    }
    if payload.item_id <= 0 {
        return Err(Error::InvalidSiteHighlight(
            "item_id must be positive".to_string(),
        ));
    }
    if payload.forum_thread_id <= 0 {
        return Err(Error::InvalidSiteHighlight(
            "forum_thread_id must be positive".to_string(),
        ));
    }

    let created = arc.pool.create_site_highlight(&payload, user.sub).await?;

    Ok(HttpResponse::Ok().json(created))
}
