use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::user::{UserClassLockStatus, UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    put,
    operation_id = "Lock/unlock user class",
    tag = "User",
    path = "/api/users/{id}/lock-class",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Successfully locked/unlocked user class"),
        (status = 403, description = "Insufficient privileges"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user_id: Path<i32>,
    form: Json<UserClassLockStatus>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(current_user.sub, &UserPermission::LockUserClass, req.path())
        .await?;

    arc.pool
        .lock_user_class(*user_id, form.class_locked)
        .await?;

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}
