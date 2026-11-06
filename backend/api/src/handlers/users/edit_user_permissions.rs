use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::user::{UpdatedUserPermissions, UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    put,
    operation_id = "Edit user permissions",
    tag = "User",
    path = "/api/users/{id}/permissions",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Successfully updated user permissions"),
        (status = 403, description = "Insufficient privileges or user class is locked"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user_id: Path<i32>,
    form: Json<UpdatedUserPermissions>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            current_user.sub,
            &UserPermission::SetAndViewUserPermissions,
            req.path(),
        )
        .await?;

    // Verify user exists and check if class is locked
    let target_user = arc.pool.find_user_with_id(*user_id).await?;

    if target_user.class_locked {
        return Err(Error::UserClassLocked);
    }

    arc.pool
        .update_user_permissions(*user_id, &form.permissions)
        .await?;

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}
