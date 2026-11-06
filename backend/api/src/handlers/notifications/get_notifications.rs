use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::notification::Notifications, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetNotificationsQuery {
    pub include_read: bool,
}

#[utoipa::path(
    get,
    operation_id = "Get notifications",
    tag = "Notification",
    path = "/api/notifications",
    params (GetNotificationsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got all notifications", body = Notifications),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetNotificationsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let notifications = arc
        .pool
        .find_all_notifications(user.sub, query.include_read)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!(notifications)))
}
