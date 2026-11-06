use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::subscription::SearchSubscriptionsQuery;
use arcadia_storage::models::title_group::TitleGroupHierarchyLite;
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
    get,
    operation_id = "Get title group comments subscriptions",
    tag = "Subscription",
    path = "/api/subscriptions/title-group-comments",
    params(SearchSubscriptionsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved title group comments subscriptions", body = PaginatedResults<TitleGroupHierarchyLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchSubscriptionsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let results = arc
        .pool
        .find_subscription_title_group_comments(user.sub, &query)
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
