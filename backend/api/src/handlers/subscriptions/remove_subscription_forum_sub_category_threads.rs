use crate::{
    Arcadia,
    handlers::subscriptions::create_subscription_forum_sub_category_threads::AddSubscriptionForumSubCategoryThreadsQuery,
    middlewares::auth_middleware::Authdata,
};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;

pub type RemoveSubscriptionForumSubCategoryThreadsQuery =
    AddSubscriptionForumSubCategoryThreadsQuery;

#[utoipa::path(
    delete,
    operation_id = "Remove forum sub-category threads subscription",
    tag = "Subscription",
    path = "/api/subscriptions/forum-sub-category-threads",
    params (RemoveSubscriptionForumSubCategoryThreadsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully unsubscribed to the item"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<RemoveSubscriptionForumSubCategoryThreadsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .delete_subscription_forum_sub_category_threads(query.forum_sub_category_id, user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"result": "success"})))
}
