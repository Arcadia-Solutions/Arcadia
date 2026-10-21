use crate::{middlewares::auth_middleware::Authdata, services::ip_check, Arcadia};
use actix_web::{web::Data, HttpRequest, HttpResponse};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CheckMissingRegistrationIpsResponse {
    pub checked_users_amount: u64,
}

#[utoipa::path(
    post,
    operation_id = "Check missing registration ips",
    tag = "Maintenance Tools",
    path = "/api/maintenance-tools/check-missing-registration-ips",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully checked and stored the registration ips that were never checked", body = CheckMissingRegistrationIpsResponse),
        (status = 503, description = "Every ip check provider failed, or none is configured. The users checked before it are stored"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::UseMaintenanceTools, req.path())
        .await?;

    let mut checked_users_amount = 0;
    for (user_id, ip) in arc.pool.find_users_without_registration_ip_check().await? {
        if !ip_check::is_public(ip.ip()) {
            continue;
        }
        // ponytail: sequential, one provider request per user, fine for the occasional backfill
        let check = arc
            .api
            .ip_check_providers
            .check_ip(&arc.http_client, ip.ip())
            .await?;
        arc.pool
            .update_user_registration_ip_check(user_id, &check)
            .await?;
        checked_users_amount += 1;
    }

    Ok(
        HttpResponse::Ok().json(CheckMissingRegistrationIpsResponse {
            checked_users_amount,
        }),
    )
}
