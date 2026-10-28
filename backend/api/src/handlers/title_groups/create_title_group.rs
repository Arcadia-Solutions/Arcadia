use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_storage::{
    models::title_group::{PublicRating, TitleGroup, UserCreatedTitleGroup},
    redis::RedisPoolInterface,
};
use futures::future::join_all;

use crate::{
    handlers::external_db::get_tmdb_data::get_tmdb_rating,
    middlewares::auth_middleware::Authdata,
    services::image_service::{validate_image_urls, validate_trailer_urls},
    Arcadia,
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
) -> Result<HttpResponse> {
    if arc.settings.lock().unwrap().uploads_disabled {
        return Err(Error::UploadsDisabled);
    }
    let approved_image_hosts = arc.settings.lock().unwrap().approved_image_hosts.clone();
    validate_image_urls(&form.covers, &approved_image_hosts)?;
    validate_image_urls(&form.screenshots, &approved_image_hosts)?;
    validate_trailer_urls(&form.trailers)?;

    let rating_futures: Vec<_> = form
        .external_links
        .iter()
        .filter(|link| link.contains("https://www.themoviedb.org/"))
        .map(|link| {
            get_tmdb_rating(
                &arc.http_client,
                link,
                arc.api.tmdb_api_key.clone().unwrap(),
            )
        })
        .collect();
    let ratings: Vec<PublicRating> = join_all(rating_futures)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();

    let created_title_group = arc
        .pool
        .create_title_group(&form, &ratings, user.sub)
        .await?;

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
            .create_artists_affiliation(
                &form.affiliated_artists,
                user.sub,
                &arc.notification_sender,
            )
            .await?;
    }

    Ok(HttpResponse::Created().json(created_title_group))
}
