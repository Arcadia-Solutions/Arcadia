use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group_review::{TitleGroupReviewHierarchy, TitleGroupReviewSearchQuery},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search title group reviews",
    tag = "Search",
    path = "/api/search/title-group-reviews",
    params (TitleGroupReviewSearchQuery),
    description = "The reviews of a title group, oldest first. The content search is case insensitive",
    responses(
        (status = 200, description = "Successfully searched the title group reviews", body=PaginatedResults<TitleGroupReviewHierarchy>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<TitleGroupReviewSearchQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    if query.minimum_rating > query.maximum_rating {
        return Err(Error::TitleGroupReviewRatingRangeInverted);
    }

    let results = arc.pool.search_title_group_reviews(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
