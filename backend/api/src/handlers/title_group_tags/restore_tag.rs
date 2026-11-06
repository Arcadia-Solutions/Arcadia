use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        title_group_tag::{RestoreTitleGroupTagRequest, TitleGroupTag},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Restore title group tag",
    tag = "Title Group Tag",
    path = "/api/title-group-tags/restore",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully restored the title group tag", body=TitleGroupTag),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: Json<RestoreTitleGroupTagRequest>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageTitleGroupTags, req.path())
        .await?;

    let restored_tag = arc.pool.restore_title_group_tag(request.id).await?;

    Ok(HttpResponse::Ok().json(restored_tag))
}
