pub mod discovery;
pub mod token;
pub mod user_status;
pub mod userinfo;

use actix_web::{
    http::header::AUTHORIZATION,
    web::{get, post, resource, ServiceConfig},
    HttpRequest, HttpResponse,
};
use arcadia_storage::redis::RedisPoolInterface;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{config::OidcConfig, services::oauth::secrets_match, Arcadia};

/// An OAuth2 error answer, shaped as `{error}`.
#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthErrorResponse {
    pub error: String,
}

impl OAuthErrorResponse {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

/// Authenticates the OAuth2 client from its `Authorization: Basic base64(client_id:client_secret)`
/// header, answering `401 invalid_client` when it is missing, unknown or has the wrong secret.
pub fn authenticate_client<'a, R: RedisPoolInterface>(
    request: &HttpRequest,
    arc: &'a Arcadia<R>,
) -> Result<&'a OidcConfig, Box<HttpResponse>> {
    basic_auth_credentials(request)
        .and_then(|(client_id, client_secret)| {
            arc.oidc.as_ref().filter(|client| {
                client.client_id == client_id
                    && secrets_match(&client_secret, &client.client_secret)
            })
        })
        .ok_or_else(|| {
            Box::new(HttpResponse::Unauthorized().json(OAuthErrorResponse::new("invalid_client")))
        })
}

fn basic_auth_credentials(request: &HttpRequest) -> Option<(String, String)> {
    let encoded = request
        .headers()
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Basic ")?;
    let credentials = String::from_utf8(BASE64_STANDARD.decode(encoded.trim()).ok()?).ok()?;
    let (client_id, client_secret) = credentials.split_once(':')?;
    Some((client_id.to_string(), client_secret.to_string()))
}

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("/.well-known/openid-configuration").route(get().to(self::discovery::exec::<R>)),
    );
    cfg.service(resource("/token").route(post().to(self::token::exec::<R>)));
    cfg.service(resource("/userinfo").route(get().to(self::userinfo::exec::<R>)));
    cfg.service(resource("/users/{user_id}/status").route(get().to(self::user_status::exec::<R>)));
}
