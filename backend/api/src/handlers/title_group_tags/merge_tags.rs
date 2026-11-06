use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        title_group_tag::{MergeTitleGroupTagsRequest, TitleGroupTag},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Merge title group tags",
    tag = "Title Group Tag",
    path = "/api/title-group-tags/merge",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully merged the source tag into the target one", body=TitleGroupTag),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: Json<MergeTitleGroupTagsRequest>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageTitleGroupTags, req.path())
        .await?;

    let target_tag = arc.pool.merge_title_group_tags(&request, user.sub).await?;

    Ok(HttpResponse::Ok().json(target_tag))
}
