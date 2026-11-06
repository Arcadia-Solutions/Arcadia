use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum::GetForumSubCategoryAllowedPostersQuery,
        user::{UserLiteAvatar, UserPermission},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get forum sub-category allowed posters",
    tag = "Forum",
    path = "/api/forum/sub-category/allowed-poster",
    params(
        ("forum_sub_category_id" = i32, Query, description = "Forum sub-category ID")
    ),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved the allowed posters", body = Vec<UserLiteAvatar>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetForumSubCategoryAllowedPostersQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditForumSubCategory, req.path())
        .await?;

    let allowed_posters = arc
        .pool
        .get_forum_sub_category_allowed_posters(query.forum_sub_category_id)
        .await?;

    Ok(HttpResponse::Ok().json(allowed_posters))
}
