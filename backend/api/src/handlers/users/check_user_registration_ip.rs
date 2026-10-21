use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Path},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{ip_check::IpCheck, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Check user registration ip",
    tag = "User",
    path = "/api/users/{id}/registration-ip/check",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Successfully checked the registration ip of the user, the result is not stored", body=IpCheck),
        (status = 400, description = "The registration ip is not a public address"),
        (status = 403, description = "Insufficient permissions"),
        (status = 503, description = "Every ip check provider failed, or none is configured"),
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

    let registration_ip = arc.pool.find_user_registration_ip(*user_id).await?;
    let check = arc
        .api
        .ip_check_providers
        .check_ip(&arc.http_client, registration_ip.registered_from_ip.ip())
        .await?;

    Ok(HttpResponse::Ok().json(check))
}
