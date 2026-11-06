use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{user::UserPermission, user_edit_change_log::DeleteUserEditChangeLogQuery},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    delete,
    operation_id = "Delete user edit change log",
    tag = "User Edit Change Logs",
    path = "/api/user-edit-change-logs",
    params(DeleteUserEditChangeLogQuery),
    responses(
        (status = 200, description = "User edit change log deleted"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    query: Query<DeleteUserEditChangeLogQuery>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::DeleteUserEditChangeLog,
            req.path(),
        )
        .await?;

    arc.pool.delete_user_edit_change_log(query.id).await?;

    Ok(HttpResponse::Ok().finish())
}
