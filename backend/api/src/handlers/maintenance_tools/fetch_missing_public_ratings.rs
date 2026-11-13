use crate::{
    Arcadia,
    middlewares::auth_middleware::Authdata,
    services::public_rating_service::{create_public_ratings_to_fetch, spawn_public_ratings_fetch},
};
use actix_web::{HttpRequest, HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct FetchMissingPublicRatingsResponse {
    /// the (title group, external link, provider) rows created, the provider never having been
    /// asked about the link
    pub created_ratings_amount: u64,
}

#[utoipa::path(
    post,
    operation_id = "Fetch missing public ratings",
    tag = "Maintenance Tools",
    path = "/api/maintenance-tools/fetch-missing-public-ratings",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Created the missing ratings of every title group's external links, they are fetched in the background", body = FetchMissingPublicRatingsResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::UseMaintenanceTools, req.path())
        .await?;

    let title_groups_external_links = arc.pool.find_title_groups_external_links().await?;
    let created_ratings_amount =
        create_public_ratings_to_fetch(&arc, &title_groups_external_links).await?;
    spawn_public_ratings_fetch(arc.clone(), None);

    Ok(HttpResponse::Ok().json(FetchMissingPublicRatingsResponse {
        created_ratings_amount,
    }))
}
