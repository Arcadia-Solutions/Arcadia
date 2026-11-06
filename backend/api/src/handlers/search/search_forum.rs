use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        forum::{ForumSearchQuery, ForumSearchResult},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search forum",
    tag = "Search",
    path = "/api/search/forum",
    params (ForumSearchQuery),
    description = "Case insensitive",
    responses(
        (status = 200, description = "Successfully got the series and some data about them", body=PaginatedResults<ForumSearchResult>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<ForumSearchQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let results = arc.pool.search_forum_threads(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
