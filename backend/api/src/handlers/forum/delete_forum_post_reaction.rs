use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{reaction::DeleteForumPostReactionQuery, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    delete,
    operation_id = "Delete forum post reaction",
    tag = "Forum",
    path = "/api/forum/post/reaction",
    params(
        ("forum_post_id" = i64, Query, description = "Forum post ID to unreact from"),
        ("emoji_id" = i32, Query, description = "Emoji ID to remove the reaction of")
    ),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully removed the reaction from the forum post"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    deleted_reaction: Query<DeleteForumPostReactionQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ReactToContent, req.path())
        .await?;

    arc.pool
        .delete_forum_post_reaction(
            deleted_reaction.forum_post_id,
            deleted_reaction.emoji_id,
            user.sub,
        )
        .await?;

    Ok(HttpResponse::Ok().finish())
}
