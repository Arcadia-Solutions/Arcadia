use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RemovedTitleGroupTag {
    pub title_group_id: i32,
    pub tag_name: String,
}

#[utoipa::path(
    delete,
    operation_id = "Remove tag from title group",
    tag = "Title Group Tag",
    path = "/api/title-group-tags/remove",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully removed the tag from the title group"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: Json<RemovedTitleGroupTag>,
    arc: Data<Arcadia<R>>,
    _: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .remove_tag_from_title_group(request.title_group_id, &request.tag_name)
        .await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
