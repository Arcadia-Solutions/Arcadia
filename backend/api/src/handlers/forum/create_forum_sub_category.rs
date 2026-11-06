use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum::{ForumSubCategory, UserCreatedForumSubCategory},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create forum sub-category",
    tag = "Forum",
    path = "/api/forum/sub-category",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the forum sub-category", body=ForumSubCategory),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    forum_sub_category: Json<UserCreatedForumSubCategory>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::CreateForumSubCategory,
            req.path(),
        )
        .await?;

    let created_sub_category = arc
        .pool
        .create_forum_sub_category(&forum_sub_category, user.sub)
        .await?;

    Ok(HttpResponse::Created().json(created_sub_category))
}
