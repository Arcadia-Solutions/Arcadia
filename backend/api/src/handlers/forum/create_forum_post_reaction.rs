use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{reaction::UserCreatedForumPostReaction, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create forum post reaction",
    tag = "Forum",
    path = "/api/forum/post/reaction",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully reacted to the forum post"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    reaction: Json<UserCreatedForumPostReaction>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ReactToContent, req.path())
        .await?;

    arc.pool
        .create_forum_post_reaction(reaction.forum_post_id, reaction.emoji_id, user.sub)
        .await?;

    Ok(HttpResponse::Created().finish())
}
