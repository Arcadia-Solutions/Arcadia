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
pub struct AppliedTitleGroupTag {
    pub title_group_id: i32,
    pub tag_id: i32,
}

#[utoipa::path(
    post,
    operation_id = "Apply tag to title group",
    tag = "Title Group Tag",
    path = "/api/title-group-tags/apply",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully applied the tag to the title group"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: Json<AppliedTitleGroupTag>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .apply_tag_to_title_group(request.title_group_id, request.tag_id, user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
