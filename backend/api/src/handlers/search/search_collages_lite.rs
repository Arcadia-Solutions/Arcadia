use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::collage::{CollageLite, SearchCollagesLiteQuery},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search collages lite",
    tag = "Search",
    path = "/api/search/collages/lite",
    params (SearchCollagesLiteQuery),
    description = "Case insensitive",
    responses(
        (status = 200, description = "Successfully got the collages lite and some data about them", body=Vec<CollageLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchCollagesLiteQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let collages = arc.pool.search_collages_lite(&query).await?;

    Ok(HttpResponse::Ok().json(collages))
}
