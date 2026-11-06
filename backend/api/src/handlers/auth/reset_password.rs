use crate::{
    Arcadia,
    services::auth_service::{validate_password, validate_password_verification},
};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserResetPassword, redis::RedisPoolInterface};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

#[utoipa::path(
    post,
    operation_id = "Reset password",
    tag = "Auth",
    path = "/api/auth/reset-password",
    request_body(content = UserResetPassword, content_type = "application/json"),
    responses(
        (status = 200, description = "Successfully reset the password"),
        (status = 400, description = "Invalid password, or invalid or expired token"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<UserResetPassword>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let user_id = arc.pool.find_password_reset_token_user(&form.token).await?;

    // the password is validated before the token is consumed, so that a rejected
    // password does not force the user to ask for a new link
    validate_password(&form.new_password)?;
    validate_password_verification(&form.new_password, &form.new_password_verify)?;

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(form.new_password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    arc.pool
        .update_user_password_hash(user_id, &password_hash)
        .await?;

    arc.pool.revoke_password_reset_token(user_id).await?;
    // whoever was logged in with the old password is logged out
    arc.auth.invalidate(user_id).await?;

    Ok(HttpResponse::Ok().finish())
}
