use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    post,
    operation_id = "Request reseed",
    tag = "Torrent",
    path = "/api/torrents/{id}/reseed-request",
    params(("id" = i32, Path, description = "Torrent id")),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Reseed requested"),
        (status = 409, description = "The torrent is not eligible for a reseed request"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i32>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::RequestReseed, req.path())
        .await?;

    let threshold_hours = arc.settings.lock().unwrap().reseed_requestable_after_hours;

    arc.pool
        .request_reseed(path.into_inner(), user.sub, threshold_hours)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
