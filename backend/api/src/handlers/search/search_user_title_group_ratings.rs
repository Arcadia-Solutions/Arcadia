use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group_rating::{TitleGroupRatingWithTitleGroup, UserTitleGroupRatingSearchQuery},
        user::{HideableUserList, ParanoiaHiddenInformation, UserPermission},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search user title group ratings",
    tag = "Search",
    path = "/api/search/title-group-ratings/user",
    params (UserTitleGroupRatingSearchQuery),
    description = "Search the title group ratings written by a user",
    responses(
        (status = 200, description = "Successfully got the title group ratings of the user", body=PaginatedResults<TitleGroupRatingWithTitleGroup>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<UserTitleGroupRatingSearchQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    // the ratings of another user are a list that their paranoia settings can hide
    if query.user_id != user.sub
        && !arc
            .pool
            .user_has_permission(user.sub, &UserPermission::SeeParanoiaHiddenUserInfo)
            .await?
        && arc
            .pool
            .find_user_paranoia_settings(query.user_id)
            .await?
            .is_list_hidden(HideableUserList::TitleGroupRatings)
    {
        return Err(Error::InsufficientPermissions(
            "this user hid this with their paranoia settings".to_string(),
        ));
    }

    let results = arc.pool.search_user_title_group_ratings(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
