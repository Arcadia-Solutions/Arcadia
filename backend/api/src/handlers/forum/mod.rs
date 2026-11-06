pub mod add_forum_sub_category_allowed_poster;
pub mod create_forum_category;
pub mod create_forum_poll;
pub mod create_forum_poll_vote;
pub mod create_forum_post;
pub mod create_forum_post_reaction;
pub mod create_forum_sub_category;
pub mod create_forum_thread;
pub mod delete_forum_category;
pub mod delete_forum_post;
pub mod delete_forum_post_reaction;
pub mod delete_forum_sub_category;
pub mod delete_forum_thread;
pub mod edit_forum_category;
pub mod edit_forum_post;
pub mod edit_forum_sub_category;
pub mod edit_forum_thread;
pub mod get_forum;
pub mod get_forum_post_reaction_users;
pub mod get_forum_sub_category_allowed_posters;
pub mod get_forum_sub_category_threads;
pub mod get_forum_thread;
pub mod get_forum_thread_posts;
pub mod pin_forum_thread;
pub mod remove_forum_sub_category_allowed_poster;
pub mod reorder_forum_category;
pub mod reorder_forum_sub_category;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(resource("").route(get().to(self::get_forum::exec::<R>)));
    cfg.service(
        resource("/category")
            .route(post().to(self::create_forum_category::exec::<R>))
            .route(put().to(self::edit_forum_category::exec::<R>))
            .route(delete().to(self::delete_forum_category::exec::<R>)),
    );
    cfg.service(
        resource("/category/reorder").route(put().to(self::reorder_forum_category::exec::<R>)),
    );
    cfg.service(
        resource("/thread")
            .route(get().to(self::get_forum_thread::exec::<R>))
            .route(put().to(self::edit_forum_thread::exec::<R>))
            .route(post().to(self::create_forum_thread::exec::<R>))
            .route(delete().to(self::delete_forum_thread::exec::<R>)),
    );
    cfg.service(resource("/poll").route(post().to(self::create_forum_poll::exec::<R>)));
    cfg.service(resource("/poll/vote").route(post().to(self::create_forum_poll_vote::exec::<R>)));
    cfg.service(resource("/thread/pin").route(put().to(self::pin_forum_thread::exec::<R>)));
    cfg.service(resource("/thread/posts").route(get().to(self::get_forum_thread_posts::exec::<R>)));
    cfg.service(
        resource("/post")
            .route(post().to(self::create_forum_post::exec::<R>))
            .route(put().to(self::edit_forum_post::exec::<R>))
            .route(delete().to(self::delete_forum_post::exec::<R>)),
    );
    cfg.service(
        resource("/post/reaction")
            .route(post().to(self::create_forum_post_reaction::exec::<R>))
            .route(delete().to(self::delete_forum_post_reaction::exec::<R>)),
    );
    cfg.service(
        resource("/post/reaction/users")
            .route(get().to(self::get_forum_post_reaction_users::exec::<R>)),
    );
    cfg.service(
        resource("/sub-category")
            .route(get().to(self::get_forum_sub_category_threads::exec::<R>))
            .route(post().to(self::create_forum_sub_category::exec::<R>))
            .route(put().to(self::edit_forum_sub_category::exec::<R>))
            .route(delete().to(self::delete_forum_sub_category::exec::<R>)),
    );
    cfg.service(
        resource("/sub-category/reorder")
            .route(put().to(self::reorder_forum_sub_category::exec::<R>)),
    );
    cfg.service(
        resource("/sub-category/allowed-poster")
            .route(get().to(self::get_forum_sub_category_allowed_posters::exec::<R>))
            .route(post().to(self::add_forum_sub_category_allowed_poster::exec::<R>))
            .route(delete().to(self::remove_forum_sub_category_allowed_poster::exec::<R>)),
    );
}
