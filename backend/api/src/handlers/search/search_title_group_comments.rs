use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group_comment::{TitleGroupCommentSearchQuery, TitleGroupCommentSearchResult},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search title group comments",
    tag = "Search",
    path = "/api/search/title-group-comments",
    params (TitleGroupCommentSearchQuery),
    description = "Case insensitive",
    responses(
        (status = 200, description = "Successfully searched title group comments", body=PaginatedResults<TitleGroupCommentSearchResult>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<TitleGroupCommentSearchQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let results = arc.pool.search_title_group_comments(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
