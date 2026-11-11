use crate::{Arcadia, middlewares::auth_middleware::Authdata};
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
    arc.validate_image_urls(artists.iter().flat_map(|artist| &artist.pictures))?;

    let artists = arc.pool.create_artists(&artists, user.sub).await?;

    Ok(HttpResponse::Created().json(artists))
}
