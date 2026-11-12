use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group_rating::{TitleGroupRatingHierarchy, TitleGroupRatingSearchQuery},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search title group ratings",
    tag = "Search",
    path = "/api/search/title-group-ratings",
    params (TitleGroupRatingSearchQuery),
    description = "The ratings of a title group. The comment search is case insensitive",
    responses(
        (status = 200, description = "Successfully searched the title group ratings", body=PaginatedResults<TitleGroupRatingHierarchy>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<TitleGroupRatingSearchQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    if query.minimum_rating > query.maximum_rating {
        return Err(Error::TitleGroupRatingRangeInverted);
    }

    let results = arc.pool.search_title_group_ratings(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
