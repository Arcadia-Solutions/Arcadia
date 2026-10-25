use crate::Arcadia;
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{models::wiki::WikiArticleResponse, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetWikiArticleQuery {
    id: i64,
}

#[utoipa::path(
    get,
    operation_id = "Get wiki article",
    tag = "Wiki",
    path = "/api/wiki/articles",
    params(GetWikiArticleQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully found the wiki article", body=WikiArticleResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetWikiArticleQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let article = arc.pool.find_wiki_article(query.id).await?;

    Ok(HttpResponse::Ok().json(article))
}
