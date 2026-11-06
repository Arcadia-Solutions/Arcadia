use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Delete all user edit change logs",
    tag = "User Edit Change Logs",
    path = "/api/user-edit-change-logs/all",
    responses(
        (status = 200, description = "All user edit change logs deleted"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::DeleteUserEditChangeLog,
            req.path(),
        )
        .await?;

    arc.pool.delete_all_user_edit_change_logs().await?;

    Ok(HttpResponse::Ok().finish())
}
