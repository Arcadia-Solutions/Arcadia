use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group_comment::{TitleGroupCommentWithLocation, UserTitleGroupCommentSearchQuery},
        user::{HideableUserList, ParanoiaHiddenInformation, UserPermission},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search user title group comments",
    tag = "Search",
    path = "/api/search/title-group-comments/user",
    params (UserTitleGroupCommentSearchQuery),
    description = "Every title group comment written by a user, most recent first",
    responses(
        (status = 200, description = "Successfully got the title group comments of the user", body=PaginatedResults<TitleGroupCommentWithLocation>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<UserTitleGroupCommentSearchQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    // the comments of another user are a list that their paranoia settings can hide
    if query.created_by_id != user.sub
        && !arc
            .pool
            .user_has_permission(user.sub, &UserPermission::SeeParanoiaHiddenUserInfo)
            .await?
        && arc
            .pool
            .find_user_paranoia_settings(query.created_by_id)
            .await?
            .is_list_hidden(HideableUserList::TitleGroupComments)
    {
        return Err(Error::InsufficientPermissions(
            "this user hid this with their paranoia settings".to_string(),
        ));
    }

    let results = arc.pool.search_user_title_group_comments(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
