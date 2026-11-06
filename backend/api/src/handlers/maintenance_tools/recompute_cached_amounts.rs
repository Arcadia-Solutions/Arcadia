use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    post,
    operation_id = "Recompute cached amounts",
    tag = "Maintenance Tools",
    path = "/api/maintenance-tools/recompute-cached-amounts",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully recomputed the cached amounts"),
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

    arc.pool.recompute_cached_amounts().await?;

    Ok(HttpResponse::Ok().finish())
}
