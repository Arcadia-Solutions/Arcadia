use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum::{ForumThread, UserCreatedForumThread},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create forum thread",
    tag = "Forum",
    path = "/api/forum/thread",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully created the forum thread", body=ForumThread),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    mut forum_thread: Json<UserCreatedForumThread>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::CreateForumThread, req.path())
        .await?;

    let forum_thread = arc
        .pool
        .create_forum_thread(&mut forum_thread, user.sub, &arc.notification_sender)
        .await?;

    Ok(HttpResponse::Created().json(forum_thread))
}
