pub mod create_collage;
pub mod create_collage_entries;
pub mod delete_collage;
pub mod delete_collage_entries;
pub mod edit_collage;
pub mod get_collage;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_collage::exec::<R>))
            .route(get().to(self::get_collage::exec::<R>))
            .route(put().to(self::edit_collage::exec::<R>))
            .route(delete().to(self::delete_collage::exec::<R>)),
    );
    cfg.service(
        resource("/entries")
            .route(post().to(self::create_collage_entries::exec::<R>))
            .route(delete().to(self::delete_collage_entries::exec::<R>)),
    );
}
