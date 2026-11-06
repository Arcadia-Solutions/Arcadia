use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::user::{GeneratedPasswordResetToken, UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create password reset token",
    tag = "User",
    path = "/api/users/{id}/password-reset-token",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 201, description = "Successfully created the password reset token", body = GeneratedPasswordResetToken),
        (status = 403, description = "Insufficient privileges"),
        (status = 404, description = "User not found"),
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
            &UserPermission::GenerateResetPasswordToken,
            req.path(),
        )
        .await?;

    let target_user_id = *user_id;
    // makes sure the user exists before handing out a token for them
    let _ = arc.pool.find_user_with_id(target_user_id).await?;

    let (token, expires_at) = arc.pool.create_password_reset_token(target_user_id).await?;

    let reset_url = format!(
        "{}/reset-password?token={}",
        arc.api.frontend_url.as_str().trim_end_matches('/'),
        token
    );

    Ok(HttpResponse::Created().json(GeneratedPasswordResetToken {
        reset_url,
        expires_at,
    }))
}
