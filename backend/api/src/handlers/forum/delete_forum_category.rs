use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{forum::DeleteForumCategoryQuery, user::UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    delete,
    operation_id = "Delete forum category",
    tag = "Forum",
    path = "/api/forum/category",
    params(
        ("id" = i32, Query, description = "Forum category ID to delete")
    ),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the forum category"),
        (status = 400, description = "Forum category has sub-categories and cannot be deleted"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    delete_request: Query<DeleteForumCategoryQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteForumCategory, req.path())
        .await?;

    arc.pool.delete_forum_category(delete_request.id).await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
