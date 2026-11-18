use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{title_group_public_rating::PublicRating, user::UserPermission},
    redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    Arcadia, middlewares::auth_middleware::Authdata,
    services::public_rating_service::refetch_public_ratings,
};

#[derive(Deserialize, ToSchema)]
pub struct RefetchTitleGroupPublicRatingsRequest {
    pub title_group_id: i32,
    /// the links of the title group to refetch the ratings of, the others are ignored
    pub external_links: Vec<String>,
}

#[utoipa::path(
    post,
    operation_id = "Refetch title group public ratings",
    tag = "Title Group",
    path = "/api/title-groups/public-ratings/refetch",
    security(
      ("http" = ["Bearer"])
    ),
    request_body = RefetchTitleGroupPublicRatingsRequest,
    responses(
        (status = 200, description = "Asked the providers for the ratings, returns the title group's ratings", body = Vec<PublicRating>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<RefetchTitleGroupPublicRatingsRequest>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::UseMaintenanceTools, req.path())
        .await?;

    let title_group = arc.pool.find_title_group(form.title_group_id).await?;

    let external_links: Vec<String> = form
        .external_links
        .iter()
        .filter(|link| title_group.external_links.contains(link))
        .cloned()
        .collect();
    refetch_public_ratings(&arc, &external_links).await?;

    let title_group = arc.pool.find_title_group(title_group.id).await?;
    Ok(HttpResponse::Ok().json(title_group.public_ratings))
}
