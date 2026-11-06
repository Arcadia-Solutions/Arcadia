use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct RemoveTitleGroupBookmarkQuery {
    pub id: i64,
}

#[utoipa::path(
    delete,
    operation_id = "Remove title group bookmark",
    tag = "Bookmark",
    path = "/api/title-group-bookmarks",
    params (RemoveTitleGroupBookmarkQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully removed the bookmark"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<RemoveTitleGroupBookmarkQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .delete_title_group_bookmark(query.id, user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"result": "success"})))
}
