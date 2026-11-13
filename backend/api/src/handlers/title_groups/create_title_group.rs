use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_storage::{
    models::{
        title_group::{TitleGroup, UserCreatedTitleGroup},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

use crate::{
    Arcadia,
    middlewares::auth_middleware::Authdata,
    services::{
        image_service::validate_trailer_urls,
        public_rating_service::sync_title_group_public_ratings,
    },
};
use arcadia_common::error::{Error, Result};

#[utoipa::path(
    post,
    operation_id = "Create title group",
    tag = "Title Group",
    path = "/api/title-groups",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully created the title_group", body=TitleGroup),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    mut form: Json<UserCreatedTitleGroup>,
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
    arc.validate_image_urls(form.covers.iter().chain(&form.screenshots))?;
    validate_trailer_urls(&form.trailers)?;

    let created_title_group = arc.pool.create_title_group(&form, user.sub).await?;

    sync_title_group_public_ratings(
        &arc,
        created_title_group.id,
        &created_title_group.external_links,
    )
    .await;

    // Subscribe the creator to the title group's comments
    arc.pool
        .create_subscription_title_group_comments(created_title_group.id, user.sub)
        .await?;

    if !form.affiliated_artists.is_empty() {
        for artist in &mut form.affiliated_artists {
            artist.title_group_id = created_title_group.id
        }

        let _ = arc
            .pool
            .create_artists_affiliation(&form.affiliated_artists, user.sub)
            .await?;
    }

    Ok(HttpResponse::Created().json(created_title_group))
}
