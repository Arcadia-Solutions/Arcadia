use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum::{ForumCategoryEnriched, ForumItemId},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetForumCategoryQuery {
    pub id: i32,
}

#[utoipa::path(
    get,
    operation_id = "Get forum category",
    tag = "Forum",
    path = "/api/forum/category",
    params(GetForumCategoryQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Returns the forum category, to edit it", body=ForumCategoryEnriched),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetForumCategoryQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditForumCategory, req.path())
        .await?;
    let item = ForumItemId::Category(query.id);
    arc.pool.ensure_forum_item_visible(user.sub, item).await?;

    let category = arc.pool.find_forum_category(query.id).await?;
    let allowed_users = arc.pool.find_forum_item_allowed_users(item).await?;
    let class_visibility = arc.pool.find_forum_item_class_visibility(item).await?;

    Ok(HttpResponse::Ok().json(ForumCategoryEnriched {
        category,
        allowed_users,
        class_visibility,
    }))
}
