use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::title_group_tag::{TitleGroupTag, UserCreatedTitleGroupTag},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create title group tag",
    tag = "Title Group Tag",
    path = "/api/title-group-tags",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the title group tag", body=TitleGroupTag),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    tag: Json<UserCreatedTitleGroupTag>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let created_tag = arc.pool.create_title_group_tag(&tag, user.sub).await?;

    Ok(HttpResponse::Created().json(created_tag))
}
