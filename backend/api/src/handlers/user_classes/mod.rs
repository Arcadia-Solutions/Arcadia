pub mod create_user_class;
pub mod delete_user_class;
pub mod edit_user_class;
pub mod get_user_classes;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::get_user_classes::exec::<R>))
            .route(post().to(self::create_user_class::exec::<R>)),
    );
    cfg.service(
        resource("/{name}")
            .route(put().to(self::edit_user_class::exec::<R>))
            .route(delete().to(self::delete_user_class::exec::<R>)),
    );
}
