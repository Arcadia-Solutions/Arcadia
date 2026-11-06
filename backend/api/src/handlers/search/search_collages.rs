use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        collage::{CollageSearchResult, SearchCollagesQuery},
        common::PaginatedResults,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search collages",
    tag = "Search",
    path = "/api/search/collages",
    params (SearchCollagesQuery),
    description = "Case insensitive",
    responses(
        (status = 200, description = "Successfully got the collages and some data about them", body=PaginatedResults<CollageSearchResult>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchCollagesQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let collages = arc.pool.search_collages(&query).await?;

    Ok(HttpResponse::Ok().json(collages))
}
