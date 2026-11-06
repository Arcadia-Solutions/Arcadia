use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{models::emoji::Emoji, redis::RedisPoolInterface};

#[utoipa::path(
    get,
    operation_id = "Get emojis",
    tag = "Emojis",
    path = "/api/emojis",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved the emojis", body=Vec<Emoji>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    _user: Authdata,
) -> Result<HttpResponse> {
    let emojis = arc.pool.find_emojis().await?;

    Ok(HttpResponse::Ok().json(emojis))
}
