use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group_rating::{SiteWideTitleGroupRatingSearchQuery, TitleGroupRatingWithTitleGroup},
        user::{HideableUserList, ParanoiaHiddenInformation, UserPermission},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search title group ratings site wide",
    tag = "Search",
    path = "/api/search/title-group-ratings",
    params (SiteWideTitleGroupRatingSearchQuery),
    description = "Search the ratings of every title group, optionally only the ones written by a user",
    responses(
        (status = 200, description = "Successfully got the title group ratings", body=PaginatedResults<TitleGroupRatingWithTitleGroup>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SiteWideTitleGroupRatingSearchQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    // the ratings of another user are a list that their paranoia settings can hide
    if let Some(user_id) = query.user_id
        && user_id != user.sub
        && !arc
            .pool
            .user_has_permission(user.sub, &UserPermission::SeeParanoiaHiddenUserInfo)
            .await?
        && arc
            .pool
            .find_user_paranoia_settings(user_id)
            .await?
            .is_list_hidden(HideableUserList::TitleGroupRatings)
    {
        return Err(Error::InsufficientPermissions(
            "this user hid this with their paranoia settings".to_string(),
        ));
    }

    let results = arc
        .pool
        .search_title_group_ratings_site_wide(&query)
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
