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
pub struct AddSubscriptionTitleGroupTorrentsQuery {
    pub title_group_id: i32,
}

#[utoipa::path(
    post,
    operation_id = "Create title group torrents subscription",
    tag = "Subscription",
    path = "/api/subscriptions/title-group-torrents",
    params (AddSubscriptionTitleGroupTorrentsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully subscribed to the item"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<AddSubscriptionTitleGroupTorrentsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .create_subscription_title_group_torrents(query.title_group_id, user.sub)
        .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({"result": "success"})))
}
