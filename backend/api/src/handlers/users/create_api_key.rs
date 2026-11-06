use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::user::{CreatedAPIKey, UserCreatedAPIKey},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create API key",
    tag = "User",
    path = "/api/users/api-keys",
    request_body(content = UserCreatedAPIKey, content_type = "application/json"),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the API key", body=CreatedAPIKey),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    body: Json<UserCreatedAPIKey>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let created_api_key = arc.pool.create_api_key(&body, user.sub).await?;

    Ok(HttpResponse::Created().json(created_api_key))
}
