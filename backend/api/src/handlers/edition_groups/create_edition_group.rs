use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        edition_group::{EditionGroup, UserCreatedEditionGroup},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create edition group",
    tag = "Edition Group",
    path = "/api/edition-groups",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully created the edition_group", body=EditionGroup),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<UserCreatedEditionGroup>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::UploadTorrent, req.path())
        .await?;

    if arc.settings.lock().unwrap().uploads_disabled {
        return Err(Error::UploadsDisabled);
    }
    arc.validate_image_urls(&form.covers)?;

    let edition_group = arc.pool.create_edition_group(&form, user.sub).await?;

    Ok(HttpResponse::Created().json(edition_group))
}
