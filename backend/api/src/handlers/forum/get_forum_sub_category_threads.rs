use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum::{ForumItemId, ForumSubCategoryHierarchy},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};
use serde::Deserialize;
use serde_json::json;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetForumSubCategoryThreadsQuery {
    id: i32,
}

#[utoipa::path(
    get,
    operation_id = "Get forum sub-category threads",
    tag = "Forum",
    path = "/api/forum/sub-category",
    params(GetForumSubCategoryThreadsQuery),
    responses(
        (status = 200, description = "Returns the threads in the forum sub-category", body=ForumSubCategoryHierarchy),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<GetForumSubCategoryThreadsQuery>,
    user: Authdata,
) -> Result<HttpResponse> {
    let mut sub_category = arc
        .pool
        .find_forum_sub_category_threads(query.id, user.sub)
        .await?;

    if arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditForumSubCategory)
        .await?
    {
        let item = ForumItemId::SubCategory(query.id);
        let allowed_users = arc.pool.find_forum_item_allowed_users(item).await?;
        let class_visibility = arc.pool.find_forum_item_class_visibility(item).await?;
        sub_category["allowed_users"] = json!(allowed_users);
        sub_category["class_visibility"] = json!(class_visibility);
    }

    Ok(HttpResponse::Ok().json(sub_category))
}
