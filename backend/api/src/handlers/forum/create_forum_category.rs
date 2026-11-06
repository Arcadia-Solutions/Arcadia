use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum::{ForumCategory, UserCreatedForumCategory},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create forum category",
    tag = "Forum",
    path = "/api/forum/category",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the forum category", body=ForumCategory),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    forum_category: Json<UserCreatedForumCategory>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::CreateForumCategory, req.path())
        .await?;

    let created_category = arc
        .pool
        .create_forum_category(&forum_category, user.sub)
        .await?;

    Ok(HttpResponse::Created().json(created_category))
}
