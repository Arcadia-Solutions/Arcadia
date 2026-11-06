use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{emoji::EmojiEnabledUpdate, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Set emoji enabled",
    tag = "Emojis",
    path = "/api/emojis/enabled",
    request_body = EmojiEnabledUpdate,
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully updated whether the emoji is enabled"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    update: Json<EmojiEnabledUpdate>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditArcadiaSettings, req.path())
        .await?;

    arc.pool
        .set_emoji_enabled(update.id, update.enabled)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
