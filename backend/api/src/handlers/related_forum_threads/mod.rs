pub mod create_related_forum_thread;
pub mod delete_related_forum_thread;

use actix_web::web::{ServiceConfig, delete, post, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_related_forum_thread::exec::<R>))
            .route(delete().to(self::delete_related_forum_thread::exec::<R>)),
    );
}
