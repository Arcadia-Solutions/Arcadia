use crate::{
    Arcadia,
    handlers::subscriptions::create_subscription_title_group_torrents::AddSubscriptionTitleGroupTorrentsQuery,
    middlewares::auth_middleware::Authdata,
};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;

pub type RemoveSubscriptionTitleGroupTorrentsQuery = AddSubscriptionTitleGroupTorrentsQuery;

#[utoipa::path(
    delete,
    operation_id = "Remove title group torrents subscription",
    tag = "Subscription",
    path = "/api/subscriptions/title-group-torrents",
    params (RemoveSubscriptionTitleGroupTorrentsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully unsubscribed to the item"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<RemoveSubscriptionTitleGroupTorrentsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .delete_subscription_title_group_torrents(query.title_group_id, user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"result": "success"})))
}
