use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::APIKey, redis::RedisPoolInterface};

#[utoipa::path(
    get,
    operation_id = "Get API keys",
    tag = "User",
    path = "/api/users/api-keys",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved the API keys of the current user", body = Vec<APIKey>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let api_keys = arc.pool.find_api_keys(user.sub).await?;

    Ok(HttpResponse::Ok().json(api_keys))
}
