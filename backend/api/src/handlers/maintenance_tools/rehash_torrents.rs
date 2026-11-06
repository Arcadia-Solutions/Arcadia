use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct RehashTorrentsResponse {
    pub updated_torrents_amount: u64,
}

#[utoipa::path(
    post,
    operation_id = "Rehash torrents with source tag",
    tag = "Maintenance Tools",
    path = "/api/maintenance-tools/rehash-torrents",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully rehashed torrents", body = RehashTorrentsResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::UseMaintenanceTools, req.path())
        .await?;

    let updated_torrents_amount = arc
        .pool
        .rehash_torrents_with_source_tag(arc.tracker.torrent_source_tag.as_deref())
        .await?;

    Ok(HttpResponse::Ok().json(RehashTorrentsResponse {
        updated_torrents_amount,
    }))
}
