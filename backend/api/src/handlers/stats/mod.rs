pub mod get_forum_stats;
pub mod get_torrent_stats;
pub mod get_user_stats;

use actix_web::web::{ServiceConfig, get, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(resource("/torrents").route(get().to(self::get_torrent_stats::exec::<R>)))
        .service(resource("/forum").route(get().to(self::get_forum_stats::exec::<R>)))
        .service(resource("/users").route(get().to(self::get_user_stats::exec::<R>)));
}
