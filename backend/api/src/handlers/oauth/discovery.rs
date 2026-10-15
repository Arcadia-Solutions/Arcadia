use actix_web::{web::Data, HttpResponse};
use arcadia_storage::redis::RedisPoolInterface;
use serde::Serialize;

use crate::Arcadia;

/// The subset of the OpenID Connect discovery document clients need for the authorization
/// code flow with userinfo. No ID token is issued, so there is no `jwks_uri`.
#[derive(Debug, Serialize)]
pub struct OpenIdConfiguration {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub response_types_supported: &'static [&'static str],
    pub grant_types_supported: &'static [&'static str],
    pub subject_types_supported: &'static [&'static str],
    pub scopes_supported: &'static [&'static str],
    pub token_endpoint_auth_methods_supported: &'static [&'static str],
    pub code_challenge_methods_supported: &'static [&'static str],
    pub claims_supported: &'static [&'static str],
}

/// OpenID Connect discovery document. The authorization endpoint is the frontend page that
/// forwards the request to the authorize endpoint with the user's credentials.
pub async fn exec<R: RedisPoolInterface + 'static>(arc: Data<Arcadia<R>>) -> HttpResponse {
    let Some(oidc) = &arc.oidc else {
        return HttpResponse::NotFound().finish();
    };
    let issuer = oidc.issuer.as_str().trim_end_matches('/');
    HttpResponse::Ok().json(OpenIdConfiguration {
        authorization_endpoint: format!(
            "{}/oauth/authorize",
            arc.api.frontend_url.as_str().trim_end_matches('/')
        ),
        token_endpoint: format!("{issuer}/token"),
        userinfo_endpoint: format!("{issuer}/userinfo"),
        issuer: issuer.to_string(),
        response_types_supported: &["code"],
        grant_types_supported: &["authorization_code"],
        subject_types_supported: &["public"],
        scopes_supported: &["openid", "profile", "email"],
        token_endpoint_auth_methods_supported: &["client_secret_basic"],
        code_challenge_methods_supported: &["S256"],
        claims_supported: &["sub", "preferred_username", "email"],
    })
}
