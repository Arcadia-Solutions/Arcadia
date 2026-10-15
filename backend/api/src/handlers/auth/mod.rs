pub mod create_user_application;
pub mod irc_auth;
pub mod login;
pub mod logout;
pub mod oauth_authorize;
pub mod refresh_token;
pub mod register;
pub mod reset_password;

use actix_web::web::{post, resource, ServiceConfig};
use arcadia_storage::redis::RedisPoolInterface;

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
