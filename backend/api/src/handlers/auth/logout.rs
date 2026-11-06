use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
    post,
    operation_id = "Logout",
    tag = "Auth",
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Successfully logged out"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: web::Data<Arcadia<R>>,
    auth: Authdata,
) -> Result<HttpResponse> {
    arc.auth.invalidate(auth.sub).await?;
    Ok(HttpResponse::Ok().finish())
}
