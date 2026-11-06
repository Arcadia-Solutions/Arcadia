use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::reaction::{ForumPostReactionUsers, GetForumPostReactionUsersQuery},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get forum post reaction users",
    tag = "Forum",
    path = "/api/forum/post/reaction/users",
    params(GetForumPostReactionUsersQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved the users who reacted", body=Vec<ForumPostReactionUsers>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetForumPostReactionUsersQuery>,
    arc: Data<Arcadia<R>>,
    _user: Authdata,
) -> Result<HttpResponse> {
    let grouped_users = arc
        .pool
        .find_forum_post_reaction_users(query.forum_post_id)
        .await?;

    Ok(HttpResponse::Ok().json(grouped_users))
}
