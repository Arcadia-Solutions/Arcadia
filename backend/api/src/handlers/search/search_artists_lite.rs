use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::artist::ArtistLite, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetArtistLiteQuery {
    name: String,
}

#[utoipa::path(
    get,
    operation_id = "Search artists lite",
    tag = "Search",
    path = "/api/search/artists/lite",
    params (GetArtistLiteQuery),
    description = "Case insensitive",
    responses(
        (status = 200, description = "Successfully got the artists and some data about them (limits to 10 results)", body=Vec<ArtistLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetArtistLiteQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let artists = arc.pool.find_artists_lite(&query.name, 10).await?;

    Ok(HttpResponse::Ok().json(artists))
}
