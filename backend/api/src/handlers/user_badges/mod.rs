pub mod award_user_badge;
pub mod create_user_badge;
pub mod create_user_badge_category;
pub mod delete_user_badge;
pub mod delete_user_badge_category;
pub mod edit_user_badge;
pub mod edit_user_badge_category;
pub mod list_user_badge_categories;
pub mod list_user_badges;
pub mod revoke_user_earned_badge;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::list_user_badges::exec::<R>))
            .route(post().to(self::create_user_badge::exec::<R>))
            .route(put().to(self::edit_user_badge::exec::<R>)),
    );
    cfg.service(resource("/award").route(post().to(self::award_user_badge::exec::<R>)));
    cfg.service(
        resource("/award/{id}").route(delete().to(self::revoke_user_earned_badge::exec::<R>)),
    );
    cfg.service(resource("/{id}").route(delete().to(self::delete_user_badge::exec::<R>)));
}

pub fn config_categories<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::list_user_badge_categories::exec::<R>))
            .route(post().to(self::create_user_badge_category::exec::<R>))
            .route(put().to(self::edit_user_badge_category::exec::<R>)),
    );
    cfg.service(resource("/{id}").route(delete().to(self::delete_user_badge_category::exec::<R>)));
}
