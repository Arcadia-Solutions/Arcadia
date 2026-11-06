use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::user::{UserClassChange, UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    put,
    operation_id = "Change user class",
    tag = "User",
    path = "/api/users/{id}/class",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Successfully changed user class"),
        (status = 403, description = "Insufficient privileges or user class is locked"),
        (status = 404, description = "User or user class not found"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user_id: Path<i32>,
    form: Json<UserClassChange>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            current_user.sub,
            &UserPermission::ChangeUserClass,
            req.path(),
        )
        .await?;

    // Verify user exists
    let target_user = arc.pool.find_user_with_id(*user_id).await?;

    // Check if user's class is locked
    if target_user.class_locked {
        return Err(Error::UserClassLocked);
    }

    // Verify target user class exists
    arc.pool.get_user_class_by_name(&form.class_name).await?;

    // Change user class
    arc.pool
        .change_user_class(*user_id, &form.class_name, true)
        .await?;

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}
