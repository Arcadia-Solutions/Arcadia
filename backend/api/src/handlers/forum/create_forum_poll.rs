use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum::{ForumPoll, UserCreatedForumPoll},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create forum poll",
    tag = "Forum",
    path = "/api/forum/poll",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the forum poll", body=ForumPoll),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    forum_poll: Json<UserCreatedForumPoll>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::CreateForumThread, req.path())
        .await?;

    let forum_poll = arc.pool.create_forum_poll(&forum_poll, user.sub).await?;

    Ok(HttpResponse::Created().json(forum_poll))
}
