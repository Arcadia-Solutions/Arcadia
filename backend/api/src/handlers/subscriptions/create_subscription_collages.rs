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
pub struct AddSubscriptionCollagesQuery {
    pub collage_id: i64,
}
#[utoipa::path(
    post,
    operation_id = "Create collage subscription",
    tag = "Subscription",
    path = "/api/subscriptions/collages",
    params(AddSubscriptionCollagesQuery),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully subscribed to the item"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<AddSubscriptionCollagesQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .create_subscription_collages(query.collage_id, user.sub)
        .await?;
    Ok(HttpResponse::Created().finish())
}
