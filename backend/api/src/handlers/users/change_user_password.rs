use crate::{
    middlewares::auth_middleware::Authdata,
    services::auth_service::{validate_password, validate_password_verification},
    Arcadia,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::user::{LoginResponse, UserChangedPassword},
    redis::RedisPoolInterface,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};

#[utoipa::path(
    put,
    operation_id = "Change user password",
    tag = "User",
    path = "/api/users/password",
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Successfully changed the user password", body = LoginResponse),
        (status = 400, description = "Invalid password or wrong current password"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<UserChangedPassword>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    // a user only ever changes their own password. staff members with the
    // `GenerateResetPasswordToken` permission hand out a password reset link instead
    let target_user_id = current_user.sub;

    let target_user = arc.pool.find_user_with_id(target_user_id).await?;

    let parsed_hash = PasswordHash::new(&target_user.password_hash)
        .map_err(|_| Error::WrongUsernameOrPassword)?;
    Argon2::default()
        .verify_password(form.current_password.as_bytes(), &parsed_hash)
        .map_err(|_| Error::WrongUsernameOrPassword)?;

    validate_password(&form.new_password)?;
    validate_password_verification(&form.new_password, &form.new_password_verify)?;

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(form.new_password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    arc.pool
        .update_user_password_hash(target_user_id, &password_hash)
        .await?;

    // every session opened with the old password is closed
    arc.auth.invalidate(target_user_id).await?;
    arc.pool.revoke_password_reset_token(target_user_id).await?;

    // the user keeps browsing with tokens issued after the invalidation instead of having to
    // log in again
    let tokens = arc
        .auth
        .generate_login_tokens(
            target_user_id,
            &arc.api.jwt_secret,
            true,
            Utc::now() + Duration::seconds(1),
        )
        .await?;

    Ok(HttpResponse::Ok().json(tokens))
}
