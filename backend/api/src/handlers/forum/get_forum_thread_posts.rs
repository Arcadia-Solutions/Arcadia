use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        forum::{ForumPostHierarchy, GetForumThreadPostsQuery},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get forum thread's posts",
    tag = "Forum",
    path = "/api/forum/thread/posts",
    params(GetForumThreadPostsQuery),
    responses(
        (status = 200, description = "Returns the thread's posts", body=PaginatedResults<ForumPostHierarchy>)
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<GetForumThreadPostsQuery>,
    user: Authdata,
) -> Result<HttpResponse> {
    //TODO: restrict access to some sub_categories based on forbidden_classes

    let thread = arc
        .pool
        .find_forum_thread_posts(query.into_inner(), user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(thread))
}
