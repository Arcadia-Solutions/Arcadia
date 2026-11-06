use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
    delete,
    operation_id = "Delete API key",
    tag = "User",
    path = "/api/users/api-keys/{id}",
    security(
      ("http" = ["Bearer"])
    ),
    params(
        ("id" = i64, Path, description = "API key ID")
    ),
    responses(
        (status = 200, description = "Successfully deleted the API key"),
        (status = 404, description = "API key not found"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    api_key_id: Path<i64>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool.delete_api_key(*api_key_id, user.sub).await?;

    Ok(HttpResponse::Ok().finish())
}
