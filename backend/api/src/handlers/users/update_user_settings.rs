use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserSettings, redis::RedisPoolInterface};
use serde_json::json;

#[utoipa::path(
    put,
    operation_id = "Update user settings",
    tag = "User",
    path = "/api/users/settings",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully updated user settings"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    settings: Json<UserSettings>,
    user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    arc.pool.update_user_settings(user.sub, &settings).await?;
    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}
