use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group_review::{TitleGroupReviewWithTitleGroup, UserTitleGroupReviewSearchQuery},
        user::{HideableUserList, ParanoiaHiddenInformation, UserPermission},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search user title group reviews",
    tag = "Search",
    path = "/api/search/title-group-reviews/user",
    params (UserTitleGroupReviewSearchQuery),
    description = "Every title group review written by a user, most recent first",
    responses(
        (status = 200, description = "Successfully got the title group reviews of the user", body=PaginatedResults<TitleGroupReviewWithTitleGroup>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<UserTitleGroupReviewSearchQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    // the reviews of another user are a list that their paranoia settings can hide
    if query.user_id != user.sub
        && !arc
            .pool
            .user_has_permission(user.sub, &UserPermission::SeeParanoiaHiddenUserInfo)
            .await?
        && arc
            .pool
            .find_user_paranoia_settings(query.user_id)
            .await?
            .is_list_hidden(HideableUserList::TitleGroupReviews)
    {
        return Err(Error::InsufficientPermissions(
            "this user hid this with their paranoia settings".to_string(),
        ));
    }

    let results = arc.pool.search_user_title_group_reviews(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
