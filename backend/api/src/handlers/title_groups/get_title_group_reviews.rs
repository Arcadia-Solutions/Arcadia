use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_storage::{
    models::title_group_review::TitleGroupReviewHierarchy, redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::Arcadia;
use arcadia_common::error::Result;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetTitleGroupReviewsQuery {
    pub title_group_id: i32,
}

#[utoipa::path(
    get,
    operation_id = "Get title group reviews",
    tag = "Title Group",
    path = "/api/title-groups/reviews",
    params(GetTitleGroupReviewsQuery),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the reviews of the title group", body=Vec<TitleGroupReviewHierarchy>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<GetTitleGroupReviewsQuery>,
) -> Result<HttpResponse> {
    let reviews = arc
        .pool
        .find_title_group_reviews(query.title_group_id)
        .await?;

    Ok(HttpResponse::Ok().json(reviews))
}
