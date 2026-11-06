pub mod get_home;

use actix_web::web::{ServiceConfig, get, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(resource("").route(get().to(self::get_home::exec::<R>)));
}
