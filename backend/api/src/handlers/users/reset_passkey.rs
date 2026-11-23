use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResetPasskeyResponse {
    pub passkey: String,
}

#[utoipa::path(
    put,
    operation_id = "Reset passkey",
    tag = "User",
    path = "/api/users/passkey",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully reset the passkey", body = ResetPasskeyResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let passkey = arc.pool.reset_passkey(user.sub).await?;

    Ok(HttpResponse::Ok().json(ResetPasskeyResponse { passkey }))
}
