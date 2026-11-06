pub mod search;

use crate::middlewares::auth_middleware;
use actix_web::web::{ServiceConfig, get, resource};
use actix_web_httpauth::middleware::HttpAuthentication;
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(resource("").route(get().to(self::search::exec::<R>)).wrap(
        HttpAuthentication::with_fn(auth_middleware::authenticate_user::<R>),
    ));
}
