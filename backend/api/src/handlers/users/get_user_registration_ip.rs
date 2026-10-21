use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Path},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{ip_check::UserRegistrationIp, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get user registration ip",
    tag = "User",
    path = "/api/users/{id}/registration-ip",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Successfully got the registration ip of the user and its stored check", body=UserRegistrationIp),
        (status = 403, description = "Insufficient permissions"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user_id: Path<i32>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            current_user.sub,
            &UserPermission::ViewRegistrationIps,
            req.path(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(arc.pool.find_user_registration_ip(*user_id).await?))
}
