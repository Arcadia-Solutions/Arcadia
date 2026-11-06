use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{emoji::DeleteEmojiQuery, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    delete,
    operation_id = "Delete emoji",
    tag = "Emojis",
    path = "/api/emojis",
    params(
        ("id" = i32, Query, description = "ID of the emoji to delete")
    ),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the emoji"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    delete_request: Query<DeleteEmojiQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditArcadiaSettings, req.path())
        .await?;

    arc.pool.delete_emoji(delete_request.id).await?;

    Ok(HttpResponse::Ok().finish())
}
