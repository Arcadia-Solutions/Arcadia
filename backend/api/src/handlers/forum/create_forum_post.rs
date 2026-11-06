use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum::{ForumPost, UserCreatedForumPost},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create forum post",
    tag = "Forum",
    path = "/api/forum/post",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully created the forum post", body=ForumPost),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    forum_post: Json<UserCreatedForumPost>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::CreateForumPost, req.path())
        .await?;

    let forum_post = arc
        .pool
        .create_forum_post(&forum_post, user.sub, &arc.notification_sender)
        .await?;

    Ok(HttpResponse::Created().json(forum_post))
}
