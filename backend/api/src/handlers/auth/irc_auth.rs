use crate::Arcadia;
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::redis::RedisPoolInterface;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct IrcAuthRequest {
    #[serde(rename = "accountName")]
    pub account_name: String,
    pub passphrase: String,
}

#[derive(Serialize, ToSchema)]
pub struct IrcAuthResponse {
    pub success: bool,
}

#[utoipa::path(
    post,
    operation_id = "IRC auth callback",
    tag = "Auth",
    path = "/api/auth/irc",
    request_body(content = IrcAuthRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Authentication result", body = IrcAuthResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    body: Json<IrcAuthRequest>,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Verify the callback token
    let expected_token = arc
        .ergo
        .auth_callback_token
        .as_ref()
        .ok_or(Error::IrcNotEnabled)?;

    let provided_token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let Some(provided_token) = provided_token else {
        return Ok(HttpResponse::Ok().json(IrcAuthResponse { success: false }));
    };

    if provided_token
        .as_bytes()
        .ct_ne(expected_token.as_bytes())
        .into()
    {
        return Ok(HttpResponse::Ok().json(IrcAuthResponse { success: false }));
    }

    let user = match arc.pool.find_user_by_username(&body.account_name).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(HttpResponse::Ok().json(IrcAuthResponse { success: false }));
        }
    };

    let Some(irc_password) = &user.irc_password else {
        return Ok(HttpResponse::Ok().json(IrcAuthResponse { success: false }));
    };

    // The passphrase may be in "username:password" format (from PASS command / login-via-pass)
    // or just the raw password (from SASL PLAIN). Handle both formats.
    let passphrase = body
        .passphrase
        .strip_prefix(&format!("{}:", body.account_name))
        .unwrap_or(&body.passphrase);

    let success: bool = passphrase.as_bytes().ct_eq(irc_password.as_bytes()).into();

    Ok(HttpResponse::Ok().json(IrcAuthResponse { success }))
}
