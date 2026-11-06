use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{forum::PinForumThread, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Pin/Unpin forum thread",
    tag = "Forum",
    path = "/api/forum/thread/pin",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully pinned/unpinned the thread"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<PinForumThread>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::PinForumThread, req.path())
        .await?;

    arc.pool.pin_forum_thread(&form).await?;

    Ok(HttpResponse::Ok().finish())
}
