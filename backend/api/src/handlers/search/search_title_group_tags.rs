use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group_tag::{SearchTitleGroupTagsQuery, TitleGroupTagEnriched},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search title group tags",
    tag = "Search",
    path = "/api/search/title-group-tags",
    params(SearchTitleGroupTagsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "List of matching tags with their names and synonyms", body=PaginatedResults<TitleGroupTagEnriched>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchTitleGroupTagsQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let results = arc.pool.search_title_group_tags(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
