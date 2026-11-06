pub mod create_gift;

use actix_web::web::{ServiceConfig, post, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(resource("").route(post().to(self::create_gift::exec::<R>)));
}
