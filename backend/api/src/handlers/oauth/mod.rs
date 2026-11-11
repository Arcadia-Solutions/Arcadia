pub mod discovery;
pub mod token;
pub mod user_status;
pub mod userinfo;

use actix_web::{
    HttpRequest, HttpResponse,
    http::header::Header,
    web::{ServiceConfig, get, post, resource},
};
use actix_web_httpauth::headers::authorization::{Authorization, Basic};
use arcadia_storage::redis::RedisPoolInterface;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{Arcadia, config::OidcConfig, services::oauth::secrets_match};

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
    Authorization::<Basic>::parse(request)
        .ok()
        .and_then(|authorization| {
            let credentials = authorization.as_ref();
            arc.oidc.as_ref().filter(|client| {
                client.client_id == credentials.user_id()
                    && credentials
                        .password()
                        .is_some_and(|secret| secrets_match(secret, &client.client_secret))
            })
        })
        .ok_or_else(|| {
            Box::new(HttpResponse::Unauthorized().json(OAuthErrorResponse::new("invalid_client")))
        })
}

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("/.well-known/openid-configuration").route(get().to(self::discovery::exec::<R>)),
    );
    cfg.service(resource("/token").route(post().to(self::token::exec::<R>)));
    cfg.service(resource("/userinfo").route(get().to(self::userinfo::exec::<R>)));
    cfg.service(resource("/users/{user_id}/status").route(get().to(self::user_status::exec::<R>)));
}
