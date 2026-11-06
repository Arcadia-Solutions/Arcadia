use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::artist::ArtistEnriched, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetArtistQuery {
    id: i64,
}

#[utoipa::path(
    get,
    operation_id = "Get artist",
    tag = "Artist",
    path = "/api/artists",
    params (GetArtistQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the artist", body=ArtistEnriched),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetArtistQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let enriched = arc.pool.find_artist_enriched(query.id, user.sub).await?;

    Ok(HttpResponse::Ok().json(enriched))
}
