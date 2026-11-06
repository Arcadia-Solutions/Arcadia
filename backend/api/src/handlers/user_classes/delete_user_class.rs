use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::user::{DeleteUserClass, UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    delete,
    operation_id = "Delete user class",
    tag = "User Class",
    path = "/api/user-classes/{name}",
    security(("http" = ["Bearer"])),
    params(
        ("name" = String, Path, description = "User class name to delete")
    ),
    request_body = DeleteUserClass,
    responses(
        (status = 200, description = "Successfully deleted user class and migrated users"),
        (status = 403, description = "Insufficient privileges"),
        (status = 404, description = "User class not found or target class not found"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    name: Path<String>,
    form: Json<DeleteUserClass>,
    user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteUserClass, req.path())
        .await?;

    // Delete user class and migrate users to target class
    arc.pool
        .delete_user_class(&name, &form.target_class_name)
        .await?;

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}
