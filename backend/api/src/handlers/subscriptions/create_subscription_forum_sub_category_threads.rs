use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct AddSubscriptionForumSubCategoryThreadsQuery {
    pub forum_sub_category_id: i32,
}

#[utoipa::path(
    post,
    operation_id = "Create forum sub-category threads subscription",
    tag = "Subscription",
    path = "/api/subscriptions/forum-sub-category-threads",
    params (AddSubscriptionForumSubCategoryThreadsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully subscribed to the item"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<AddSubscriptionForumSubCategoryThreadsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .create_subscription_forum_sub_category_threads(query.forum_sub_category_id, user.sub)
        .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({"result": "success"})))
}
