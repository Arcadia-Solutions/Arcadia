use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::series::SeriesEnriched, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetSeriesQuery {
    pub id: i64,
}

#[utoipa::path(
    get,
    operation_id = "Get series",
    tag = "Series",
    path = "/api/series",
    params (GetSeriesQuery),
    responses(
        (status = 200, description = "Successfully got the series", body=SeriesEnriched),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<GetSeriesQuery>,
) -> Result<HttpResponse> {
    let enriched = arc.pool.find_series_enriched(query.id).await?;

    Ok(HttpResponse::Ok().json(enriched))
}
