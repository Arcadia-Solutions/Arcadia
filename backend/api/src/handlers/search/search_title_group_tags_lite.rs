use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{common::PaginatedResults, title_group_tag::TitleGroupTagLite},
    redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct SearchTitleGroupTagsLiteQuery {
    pub name: String,
    pub page: u32,
    pub page_size: u32,
}

#[utoipa::path(
    get,
    operation_id = "Search title group tags lite",
    tag = "Search",
    path = "/api/search/title-group-tags/lite",
    params(SearchTitleGroupTagsLiteQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "List of matching tags with their names and synonyms", body=PaginatedResults<TitleGroupTagLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchTitleGroupTagsLiteQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let results = arc
        .pool
        .search_title_group_tags_lite(&query.name, query.page, query.page_size)
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
