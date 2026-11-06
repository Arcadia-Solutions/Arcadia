use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{title_group_tag::DeleteTitleGroupTagRequest, user::UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    delete,
    operation_id = "Delete title group tag",
    tag = "Title Group Tag",
    path = "/api/title-group-tags",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the title group tag"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: Json<DeleteTitleGroupTagRequest>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageTitleGroupTags, req.path())
        .await?;

    arc.pool.delete_title_group_tag(&request, user.sub).await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
