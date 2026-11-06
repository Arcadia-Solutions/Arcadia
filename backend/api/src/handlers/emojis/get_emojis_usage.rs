use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{emoji::EmojiUsage, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get emojis usage",
    tag = "Emojis",
    path = "/api/emojis/usage",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved the amount of reactions per emoji", body=Vec<EmojiUsage>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditArcadiaSettings, req.path())
        .await?;

    let usage = arc.pool.find_emojis_usage().await?;

    Ok(HttpResponse::Ok().json(usage))
}
