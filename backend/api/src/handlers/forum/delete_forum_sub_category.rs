use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{forum::DeleteForumSubCategoryQuery, user::UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    delete,
    operation_id = "Delete forum sub-category",
    tag = "Forum",
    path = "/api/forum/sub-category",
    params(
        ("id" = i32, Query, description = "Forum sub-category ID to delete")
    ),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the forum sub-category"),
        (status = 400, description = "Forum sub-category has threads and cannot be deleted"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    delete_request: Query<DeleteForumSubCategoryQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::DeleteForumSubCategory,
            req.path(),
        )
        .await?;

    arc.pool
        .delete_forum_sub_category(delete_request.id)
        .await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
