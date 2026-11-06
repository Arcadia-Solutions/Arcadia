pub mod create_artists;
pub mod delete_artist;
pub mod edit_artist;
pub mod get_artist;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_artists::exec::<R>))
            .route(get().to(self::get_artist::exec::<R>))
            .route(put().to(self::edit_artist::exec::<R>))
            .route(delete().to(self::delete_artist::exec::<R>)),
    );
}
