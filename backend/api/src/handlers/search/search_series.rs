use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::series::{SearchSeriesQuery, SeriesSearchResponse},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search series",
    tag = "Search",
    path = "/api/search/series",
    params (SearchSeriesQuery),
    description = "Case insensitive",
    responses(
        (status = 200, description = "Successfully got the series and some data about them", body=SeriesSearchResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchSeriesQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let series = arc.pool.search_series(&query).await?;

    Ok(HttpResponse::Ok().json(series))
}
