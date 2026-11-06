use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::user::{UpdateUserCustomTitle, UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Set user custom title",
    tag = "User",
    path = "/api/users/{id}/custom-title",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Successfully updated user custom title"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user_id: Path<i32>,
    form: Json<UpdateUserCustomTitle>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            current_user.sub,
            &UserPermission::SetUserCustomTitle,
            req.path(),
        )
        .await?;

    arc.pool
        .update_user_custom_title(*user_id, form.custom_title.as_deref())
        .await?;

    Ok(HttpResponse::Ok().finish())
}
