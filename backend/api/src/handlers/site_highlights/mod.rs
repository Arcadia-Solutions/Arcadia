pub mod create_site_highlight;
pub mod delete_site_highlight;
pub mod edit_site_highlight;
pub mod list_site_highlights;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::list_site_highlights::exec::<R>))
            .route(post().to(self::create_site_highlight::exec::<R>)),
    );
    cfg.service(
        resource("/{id}")
            .route(put().to(self::edit_site_highlight::exec::<R>))
            .route(delete().to(self::delete_site_highlight::exec::<R>)),
    );
}
