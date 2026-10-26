pub mod apply_tag;
pub mod create_tag;
pub mod delete_tag;
pub mod edit_tag;
pub mod merge_tags;
pub mod remove_tag;
pub mod restore_tag;
pub mod search_applied_tags;

use actix_web::web::{delete, get, post, put, resource, ServiceConfig};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_tag::exec::<R>))
            .route(put().to(self::edit_tag::exec::<R>))
            .route(delete().to(self::delete_tag::exec::<R>)),
    );
    cfg.service(resource("/apply").route(post().to(self::apply_tag::exec::<R>)));
    cfg.service(resource("/remove").route(delete().to(self::remove_tag::exec::<R>)));
    cfg.service(resource("/restore").route(put().to(self::restore_tag::exec::<R>)));
    cfg.service(resource("/applied").route(get().to(self::search_applied_tags::exec::<R>)));
    cfg.service(resource("/merge").route(post().to(self::merge_tags::exec::<R>)));
}
