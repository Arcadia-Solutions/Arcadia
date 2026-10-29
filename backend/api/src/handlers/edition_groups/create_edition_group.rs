use crate::{
    middlewares::auth_middleware::Authdata, services::image_service::validate_image_urls, Arcadia,
};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
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
    let approved_image_hosts = arc.settings.lock().unwrap().approved_image_hosts.clone();
    validate_image_urls(&form.covers, &approved_image_hosts)?;

    let edition_group = arc.pool.create_edition_group(&form, user.sub).await?;

    Ok(HttpResponse::Created().json(edition_group))
}
