use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{forum::ReorderForumSubCategories, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Reorder forum sub-categories",
    tag = "Forum",
    path = "/api/forum/sub-category/reorder",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully reordered the forum sub-categories"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    reorder: Json<ReorderForumSubCategories>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditForumSubCategory, req.path())
        .await?;
    let sub_category_ids: Vec<i32> = reorder
        .sub_categories
        .iter()
        .map(|entry| entry.id)
        .collect();
    arc.pool
        .ensure_forum_items_visible(user.sub, &[reorder.forum_category_id], &sub_category_ids)
        .await?;

    arc.pool.reorder_forum_sub_categories(&reorder).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"result": "success"})))
}
