use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::models::collage::CollageLite;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::subscription::SearchSubscriptionsQuery;
use arcadia_storage::redis::RedisPoolInterface;
#[utoipa::path(
    get,
    operation_id = "Get collage subscriptions",
    tag = "Subscription",
    path = "/api/subscriptions/collages",
    params(SearchSubscriptionsQuery),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (
            status = 200,
            description = "Successfully retrieved collage subscriptions",
            body = PaginatedResults<CollageLite>
        ),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchSubscriptionsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let results = arc
        .pool
        .find_subscription_collages(user.sub, &query)
        .await?;
    Ok(HttpResponse::Ok().json(results))
}
