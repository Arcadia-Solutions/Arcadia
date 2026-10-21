use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json, Path},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{ip_check::IpCheck, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Set user registration ip check",
    tag = "User",
    path = "/api/users/{id}/registration-ip",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    request_body = IpCheck,
    responses(
        (status = 200, description = "Successfully stored the check of the registration ip of the user"),
        (status = 403, description = "Insufficient permissions"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user_id: Path<i32>,
    check: Json<IpCheck>,
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

    arc.pool
        .update_user_registration_ip_check(*user_id, &check)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
