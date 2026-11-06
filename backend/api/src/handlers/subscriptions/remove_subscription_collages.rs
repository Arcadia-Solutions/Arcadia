use crate::{
    Arcadia, handlers::subscriptions::create_subscription_collages::AddSubscriptionCollagesQuery,
    middlewares::auth_middleware::Authdata,
};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
pub type RemoveSubscriptionCollagesQuery = AddSubscriptionCollagesQuery;
#[utoipa::path(
    delete,
    operation_id = "Remove collage subscription",
    tag = "Subscription",
    path = "/api/subscriptions/collages",
    params(RemoveSubscriptionCollagesQuery),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (
            status = 200,
            description = "Successfully unsubscribed to the item"
        ),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<RemoveSubscriptionCollagesQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .delete_subscription_collages(query.collage_id, user.sub)
        .await?;
    Ok(HttpResponse::Ok().finish())
}
