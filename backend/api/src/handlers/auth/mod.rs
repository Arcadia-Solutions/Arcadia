pub mod create_user_application;
pub mod irc_auth;
pub mod login;
pub mod logout;
pub mod oauth_authorize;
pub mod refresh_token;
pub mod register;
pub mod reset_password;

use actix_web::{
    web::{post, resource, ServiceConfig},
    HttpRequest,
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{redis::RedisPoolInterface, sqlx::types::ipnetwork::IpNetwork};

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(resource("/register").route(post().to(self::register::exec::<R>)));
    cfg.service(resource("/login").route(post().to(self::login::exec::<R>)));
    cfg.service(resource("/logout").route(post().to(self::logout::exec::<R>)));
    cfg.service(resource("/refresh-token").route(post().to(self::refresh_token::exec::<R>)));
    cfg.service(resource("/apply").route(post().to(self::create_user_application::exec::<R>)));
    cfg.service(resource("/oauth-authorize").route(post().to(self::oauth_authorize::exec::<R>)));
    cfg.service(resource("/irc").route(post().to(self::irc_auth::exec::<R>)));
    cfg.service(resource("/reset-password").route(post().to(self::reset_password::exec::<R>)));
}

/// Ip address of the client. Forwarding headers are only trusted when the reverse proxy header
/// is configured, otherwise anyone could pick the ip address stored for them.
pub(crate) fn client_ip(
    req: &HttpRequest,
    reverse_proxy_header: Option<&str>,
) -> Result<IpNetwork> {
    match reverse_proxy_header {
        Some(header_name) => req
            .headers()
            .get(header_name)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next_back())
            .and_then(|s| s.trim().parse().ok()),
        None => req.peer_addr().map(|addr| addr.ip().into()),
    }
    .ok_or(Error::ClientIpNotFound)
}
