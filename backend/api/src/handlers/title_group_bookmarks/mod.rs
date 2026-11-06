pub mod create_title_group_bookmark;
pub mod edit_title_group_bookmark;
pub mod get_title_group_bookmark;
pub mod remove_title_group_bookmark;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_title_group_bookmark::exec::<R>))
            .route(get().to(self::get_title_group_bookmark::exec::<R>))
            .route(put().to(self::edit_title_group_bookmark::exec::<R>))
            .route(delete().to(self::remove_title_group_bookmark::exec::<R>)),
    );
}
