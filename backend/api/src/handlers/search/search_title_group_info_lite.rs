use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_storage::{
    models::title_group::{ContentType, TitleGroupLite},
    redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::Arcadia;
use arcadia_common::error::Result;

#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchTitleGroupLiteQuery {
    name: String,
    content_type: Option<ContentType>,
}

#[utoipa::path(
    get,
    operation_id = "Search title group info",
    tag = "Search",
    path = "/api/search/title-groups/lite",
    params(SearchTitleGroupLiteQuery),
    responses(
        (status = 200, description = "Returns title groups with their name containing the provided string. Searches in title group names and series names. Returns up to 5 matches for regular searches, or up to 50 matches when a series name matches (to show all seasons)", body=Vec<TitleGroupLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<SearchTitleGroupLiteQuery>,
) -> Result<HttpResponse> {
    let title_groups = arc
        .pool
        .find_title_group_info_lite(None, Some(&query.name), &query.content_type, 5)
        .await?;

    Ok(HttpResponse::Ok().json(title_groups))
}
