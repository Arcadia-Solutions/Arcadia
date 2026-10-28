use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
use serde::{Deserialize, Serialize};
use url::form_urlencoded;
use utoipa::ToSchema;

use crate::{
    handlers::oauth::OAuthErrorResponse,
    middlewares::auth_middleware::Authdata,
    services::oauth::{store_with_new_token, AuthorizationCode},
    Arcadia,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct OAuthAuthorize {
    /// Query string of the authorization request, as the frontend authorize page received it.
    pub authorization_query: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthAuthorizeResponse {
    /// Where to send the browser: the client's redirect URI with a code, or with an error.
    pub redirect_url: String,
}

/// Raw authorization request. Every field is optional so that malformed requests answer
/// with a proper OAuth2 error redirect. The scope is ignored: every claim is always granted.
#[derive(Debug, Default, Deserialize)]
struct AuthorizationRequest {
    client_id: Option<String>,
    redirect_uri: Option<String>,
    response_type: Option<String>,
    state: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
}

/// The redirect URI with the given parameters, plus `state` when the client sent one.
fn redirect_location(
    redirect_uri: &str,
    state: Option<&str>,
    parameters: &[(&str, &str)],
) -> String {
    let query = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(parameters)
        .extend_pairs(state.map(|state| ("state", state)))
        .finish();
    let separator = if redirect_uri.contains('?') { '&' } else { '?' };
    format!("{redirect_uri}{separator}{query}")
}

#[utoipa::path(
    post,
    operation_id = "OAuth authorize",
    tag = "OAuth",
    path = "/api/auth/oauth-authorize",
    description = "The frontend page the client sends the browser to (the discovery document's \
        authorization endpoint) forwards the authorization request here with the user's credentials.",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Redirect URL of the client", body = OAuthAuthorizeResponse),
        (status = 400, description = "Unknown client or redirect URI", body = OAuthErrorResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    authorization: Json<OAuthAuthorize>,
    user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let request: AuthorizationRequest =
        serde_urlencoded::from_str(&authorization.authorization_query).unwrap_or_default();

    // without a known client and its redirect URI, no redirect can be trusted
    let Some(client) = arc.oidc.as_ref().filter(|client| {
        request.client_id.as_ref() == Some(&client.client_id)
            && request.redirect_uri.as_ref() == Some(&client.redirect_uri)
    }) else {
        return Ok(HttpResponse::BadRequest().json(OAuthErrorResponse::new("invalid_request")));
    };

    let redirect = |parameters: &[(&str, &str)]| {
        Ok(HttpResponse::Ok().json(OAuthAuthorizeResponse {
            redirect_url: redirect_location(
                &client.redirect_uri,
                request.state.as_deref(),
                parameters,
            ),
        }))
    };
    if request.response_type.as_deref() != Some("code") {
        return redirect(&[
            ("error", "unsupported_response_type"),
            (
                "error_description",
                "only the authorization code flow is supported",
            ),
        ]);
    }
    let Some(code_challenge) = request
        .code_challenge
        .clone()
        // an S256 challenge is always 43 base64url characters, anything else is refused
        // before it is stored
        .filter(|challenge| {
            challenge.len() == 43
                && challenge
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        })
        .filter(|_| request.code_challenge_method.as_deref() == Some("S256"))
    else {
        return redirect(&[
            ("error", "invalid_request"),
            (
                "error_description",
                "a PKCE S256 code challenge is required",
            ),
        ]);
    };

    // the client is configured by the admins, so it is trusted: the user is not asked to consent
    let code = store_with_new_token(
        &*arc.redis_pool,
        "code",
        &AuthorizationCode {
            user_id: user.sub,
            code_challenge,
        },
    )
    .await?;

    redirect(&[("code", &code)])
}
