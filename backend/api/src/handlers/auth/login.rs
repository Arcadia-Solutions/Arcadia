use crate::Arcadia;
use actix_web::{web, HttpResponse};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::user::{Login, LoginResponse},
    redis::RedisPoolInterface,
};
use chrono::prelude::Utc;

#[utoipa::path(
    post,
    operation_id = "Login",
    tag = "Auth",
    path = "/api/auth/login",
    responses(
        (status = 200, description = "Successfully logged in", body=LoginResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: web::Data<Arcadia<R>>,
    user_login: web::Json<Login>,
) -> Result<HttpResponse> {
    let user = arc.pool.find_user_with_password(&user_login).await?;

    if user.banned {
        log::info!("Banned user tried to log in: {}", user_login.username);
        return Err(Error::AccountBanned);
    }

    let tokens = arc
        .auth
        .generate_login_tokens(
            user.id,
            &arc.api.jwt_secret,
            user_login.remember_me,
            Utc::now(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(tokens))
}
