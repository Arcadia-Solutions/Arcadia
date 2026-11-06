use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::title_group_bookmark::TitleGroupBookmark, redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetTitleGroupBookmarkQuery {
    id: i64,
}

#[utoipa::path(
    get,
    operation_id = "Get title group bookmark",
    tag = "Bookmark",
    params(GetTitleGroupBookmarkQuery),
    path = "/api/title-group-bookmarks",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Found the bookmark", body=TitleGroupBookmark),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetTitleGroupBookmarkQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let bookmark = arc
        .pool
        .find_title_group_bookmark(query.id, user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(bookmark))
}
