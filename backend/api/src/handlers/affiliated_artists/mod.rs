pub mod create_affiliated_artists;
pub mod remove_affiliated_artists;

use actix_web::web::{ServiceConfig, delete, post, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_affiliated_artists::exec::<R>))
            .route(delete().to(self::remove_affiliated_artists::exec::<R>)),
    );
}
