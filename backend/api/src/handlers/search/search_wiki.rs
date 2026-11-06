use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        wiki::{SearchWikiQuery, WikiSearchResult},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search wiki articles",
    tag = "Search",
    path = "/api/search/wiki",
    params(SearchWikiQuery),
    description = "Case insensitive wiki article search. Can search title only or title and body.",
    responses(
        (status = 200, description = "Successfully got the wiki articles", body = PaginatedResults<WikiSearchResult>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchWikiQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let articles = arc.pool.search_wiki_articles(&query).await?;
    Ok(HttpResponse::Ok().json(articles))
}
