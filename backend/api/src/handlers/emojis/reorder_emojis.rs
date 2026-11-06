use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{emoji::ReorderEmojis, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Reorder emojis",
    tag = "Emojis",
    path = "/api/emojis/reorder",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully reordered the emojis"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    reorder: Json<ReorderEmojis>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditArcadiaSettings, req.path())
        .await?;

    arc.pool.reorder_emojis(&reorder).await?;

    Ok(HttpResponse::Ok().finish())
}
