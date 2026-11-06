use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::series::SeriesLite, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SearchSeriesLiteQuery {
    pub name: String,
}

#[utoipa::path(
    get,
    operation_id = "Search series lite",
    tag = "Search",
    path = "/api/search/series/lite",
    params (SearchSeriesLiteQuery),
    description = "Case insensitive",
    responses(
        (status = 200, description = "Successfully got the series (lite)", body=Vec<SeriesLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchSeriesLiteQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let series = arc.pool.search_series_lite(&query.name, 7).await?;

    Ok(HttpResponse::Ok().json(series))
}
