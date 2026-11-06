use crate::{
    Arcadia, middlewares::auth_middleware::Authdata, services::image_service::validate_image_urls,
};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::artist::{Artist, UserCreatedArtist},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create artists",
    tag = "Artist",
    path = "/api/artists",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the artists, returned in the same order as the one sent.
            In the case of a db conflict (duplicate), the existing entry is returned (can be seen with the created_at attribute).", body=Vec<Artist>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    artists: Json<Vec<UserCreatedArtist>>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let approved_image_hosts = arc.settings.lock().unwrap().approved_image_hosts.clone();
    for artist in artists.iter() {
        validate_image_urls(&artist.pictures, &approved_image_hosts)?;
    }

    let artists = arc.pool.create_artists(&artists, user.sub).await?;

    Ok(HttpResponse::Created().json(artists))
}
