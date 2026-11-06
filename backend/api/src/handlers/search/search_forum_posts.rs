use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        forum::{ForumPostSearchQuery, ForumPostWithLocation},
        user::{HideableUserList, ParanoiaHiddenInformation, UserPermission},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search forum posts",
    tag = "Search",
    path = "/api/search/forum/posts",
    params (ForumPostSearchQuery),
    description = "Every forum post written by a user, most recent first",
    responses(
        (status = 200, description = "Successfully got the forum posts of the user", body=PaginatedResults<ForumPostWithLocation>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<ForumPostSearchQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    // the posts of another user are a list that their paranoia settings can hide
    if query.created_by_id != user.sub
        && !arc
            .pool
            .user_has_permission(user.sub, &UserPermission::SeeParanoiaHiddenUserInfo)
            .await?
        && arc
            .pool
            .find_user_paranoia_settings(query.created_by_id)
            .await?
            .is_list_hidden(HideableUserList::ForumPosts)
    {
        return Err(Error::InsufficientPermissions(
            "this user hid this with their paranoia settings".to_string(),
        ));
    }

    let results = arc.pool.search_forum_posts(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
