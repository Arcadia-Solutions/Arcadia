pub mod create_conversation;
pub mod create_conversation_message;
pub mod create_mass_conversation;
pub mod get_conversation;

use actix_web::web::{ServiceConfig, get, post, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_conversation::exec::<R>))
            .route(get().to(self::get_conversation::exec::<R>)),
    );

    cfg.service(
        resource("/messages").route(post().to(self::create_conversation_message::exec::<R>)),
    );

    cfg.service(resource("/mass").route(post().to(self::create_mass_conversation::exec::<R>)));
}
