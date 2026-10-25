use actix_web::{web, HttpRequest, HttpResponse};
use arcadia_storage::redis::RedisPoolInterface;
use serde::{Deserialize, Serialize};

use crate::{
    services::oauth::{
        pkce_challenge_matches, store_with_new_token, take_by_token, AuthorizationCode,
        OAUTH_TOKEN_LIFETIME_SECONDS,
    },
    Arcadia,
};

use super::{authenticate_client, OAuthErrorResponse};

/// Raw `application/x-www-form-urlencoded` token request. Every field is optional so that
/// malformed requests answer with a proper OAuth2 JSON error instead of the plain-text error
/// of the form extractor.
#[derive(Debug, Deserialize)]
struct TokenRequest {
    grant_type: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    code_verifier: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

/// Exchanges an authorization code for an access token. The client authenticates with HTTP Basic.
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: HttpRequest,
    body: web::Bytes,
    arc: web::Data<Arcadia<R>>,
) -> HttpResponse {
    let client = match authenticate_client(&request, &arc) {
        Ok(client) => client,
        Err(response) => return *response,
    };
    let Ok(token_request) = serde_urlencoded::from_bytes::<TokenRequest>(&body) else {
        return HttpResponse::BadRequest().json(OAuthErrorResponse::new("invalid_request"));
    };
    if token_request.grant_type.as_deref() != Some("authorization_code") {
        return HttpResponse::BadRequest().json(OAuthErrorResponse::new("unsupported_grant_type"));
    }

    let invalid_grant =
        || HttpResponse::BadRequest().json(OAuthErrorResponse::new("invalid_grant"));
    let Some(code) = token_request.code.as_deref() else {
        return invalid_grant();
    };
    let Ok(Some(authorization_code)) =
        take_by_token::<_, AuthorizationCode>(&*arc.redis_pool, "code", code).await
    else {
        return invalid_grant();
    };
    let code_verifier = token_request.code_verifier.as_deref().unwrap_or_default();
    if token_request.redirect_uri.as_ref() != Some(&client.redirect_uri)
        || !pkce_challenge_matches(code_verifier, &authorization_code.code_challenge)
    {
        return invalid_grant();
    }

    // userinfo checks the ban when the token is used, which happens right after this
    let Ok(access_token) = store_with_new_token(
        &*arc.redis_pool,
        "access_token",
        &authorization_code.user_id,
    )
    .await
    else {
        return HttpResponse::InternalServerError().json(OAuthErrorResponse::new("server_error"));
    };

    HttpResponse::Ok().json(TokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in: OAUTH_TOKEN_LIFETIME_SECONDS as i64,
    })
}
