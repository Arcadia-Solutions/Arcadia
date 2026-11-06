pub mod create_donation;
pub mod delete_donation;
pub mod edit_donation;
pub mod search_donations;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::search_donations::exec::<R>))
            .route(post().to(self::create_donation::exec::<R>))
            .route(put().to(self::edit_donation::exec::<R>))
            .route(delete().to(self::delete_donation::exec::<R>)),
    );
}
