use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_storage::{
    models::title_group_bookmark::{EditedTitleGroupBookmark, TitleGroupBookmark},
    redis::RedisPoolInterface,
};

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::Result;

#[utoipa::path(
    put,
    operation_id = "Edit title group bookmark",
    tag = "Bookmark",
    path = "/api/title-group-bookmarks",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the bookmark", body=TitleGroupBookmark),
    )
)]

pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<EditedTitleGroupBookmark>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let bookmark = arc
        .pool
        .find_title_group_bookmark(form.id, user.sub)
        .await?;

    let updated_bookmark = arc
        .pool
        .update_title_group_bookmark(&form, bookmark.id, user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(updated_bookmark))
}
