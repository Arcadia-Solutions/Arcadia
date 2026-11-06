use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::title_group_bookmark::{TitleGroupBookmark, UserCreatedTitleGroupBookmark},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create title group bookmark",
    tag = "Bookmark",
    path = "/api/title-group-bookmarks",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the bookmark", body=TitleGroupBookmark),)
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    bookmark: Json<UserCreatedTitleGroupBookmark>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let bookmark = arc
        .pool
        .create_title_group_bookmark(&bookmark, user.sub)
        .await?;

    Ok(HttpResponse::Created().json(bookmark))
}
