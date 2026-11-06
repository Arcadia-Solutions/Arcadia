use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::forum::ForumSubCategoryLite;
use arcadia_storage::models::subscription::SearchSubscriptionsQuery;
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
    get,
    operation_id = "Get forum sub-category threads subscriptions",
    tag = "Subscription",
    path = "/api/subscriptions/forum-sub-category-threads",
    params(SearchSubscriptionsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved forum sub-category threads subscriptions", body = PaginatedResults<ForumSubCategoryLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchSubscriptionsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let results = arc
        .pool
        .find_subscription_forum_sub_category_threads(user.sub, &query)
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
