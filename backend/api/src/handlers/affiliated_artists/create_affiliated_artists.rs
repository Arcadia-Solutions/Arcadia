use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::artist::{AffiliatedArtistHierarchy, UserCreatedAffiliatedArtist},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create artist affiliation",
    tag = "Affiliated Artist",
    path = "/api/affiliated-artists",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully created the artist affiliations", body=Vec<AffiliatedArtistHierarchy>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    artists: Json<Vec<UserCreatedAffiliatedArtist>>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let title_group_ids = artists
        .iter()
        .map(|artist| artist.title_group_id)
        .collect::<Vec<_>>();
    arc.pool
        .require_title_groups_edit_rights(user.sub, &title_group_ids, req.path())
        .await?;

    let affiliations = arc
        .pool
        .create_artists_affiliation(&artists, user.sub, &arc.notification_sender)
        .await?;

    Ok(HttpResponse::Created().json(affiliations))
}
