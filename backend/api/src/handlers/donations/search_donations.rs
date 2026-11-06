use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        donation::{SearchDonationsQuery, SearchDonationsResponse},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search donations",
    tag = "Donation",
    path = "/api/donations",
    security(
        ("http" = ["Bearer"])
    ),
    params(SearchDonationsQuery),
    responses(
        (status = 200, description = "Successfully retrieved donations with aggregates", body=SearchDonationsResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchDonationsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::SearchDonation, req.path())
        .await?;

    let donations = arc.pool.search_donations(&query).await?;

    Ok(HttpResponse::Ok().json(donations))
}
