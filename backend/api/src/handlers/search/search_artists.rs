use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        artist::{ArtistSearchResult, SearchArtistsQuery},
        common::PaginatedResults,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search artists",
    tag = "Search",
    path = "/api/search/artists",
    params(SearchArtistsQuery),
    description = "Case insensitive",
    responses(
        (status = 200, description = "Successfully got the artists", body = PaginatedResults<ArtistSearchResult>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchArtistsQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let artists = arc.pool.search_artists(&query).await?;
    Ok(HttpResponse::Ok().json(artists))
}
