pub mod create_subscription_artist_title_groups;
pub mod create_subscription_collages;
pub mod create_subscription_forum_sub_category_threads;
pub mod create_subscription_forum_thread_posts;
pub mod create_subscription_title_group_comments;
pub mod create_subscription_title_group_torrents;
pub mod create_subscription_torrent_request_comments;
pub mod get_subscription_artist_title_groups;
pub mod get_subscription_collages;
pub mod get_subscription_forum_sub_category_threads;
pub mod get_subscription_forum_thread_posts;
pub mod get_subscription_title_group_comments;
pub mod get_subscription_title_group_torrents;
pub mod get_subscription_torrent_request_comments;
pub mod remove_subscription_artist_title_groups;
pub mod remove_subscription_collages;
pub mod remove_subscription_forum_sub_category_threads;
pub mod remove_subscription_forum_thread_posts;
pub mod remove_subscription_title_group_comments;
pub mod remove_subscription_title_group_torrents;
pub mod remove_subscription_torrent_request_comments;
use actix_web::web::{ServiceConfig, delete, get, post, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("/forum-sub-category-threads")
            .route(get().to(self::get_subscription_forum_sub_category_threads::exec::<R>))
            .route(post().to(self::create_subscription_forum_sub_category_threads::exec::<R>))
            .route(delete().to(self::remove_subscription_forum_sub_category_threads::exec::<R>)),
    );
    cfg.service(
        resource("/forum-thread-posts")
            .route(get().to(self::get_subscription_forum_thread_posts::exec::<R>))
            .route(post().to(self::create_subscription_forum_thread_posts::exec::<R>))
            .route(delete().to(self::remove_subscription_forum_thread_posts::exec::<R>)),
    );
    cfg.service(
        resource("/title-group-torrents")
            .route(get().to(self::get_subscription_title_group_torrents::exec::<R>))
            .route(post().to(self::create_subscription_title_group_torrents::exec::<R>))
            .route(delete().to(self::remove_subscription_title_group_torrents::exec::<R>)),
    );
    cfg.service(
        resource("/title-group-comments")
            .route(get().to(self::get_subscription_title_group_comments::exec::<R>))
            .route(post().to(self::create_subscription_title_group_comments::exec::<R>))
            .route(delete().to(self::remove_subscription_title_group_comments::exec::<R>)),
    );
    cfg.service(
        resource("/artist-title-groups")
            .route(get().to(self::get_subscription_artist_title_groups::exec::<R>))
            .route(post().to(self::create_subscription_artist_title_groups::exec::<R>))
            .route(delete().to(self::remove_subscription_artist_title_groups::exec::<R>)),
    );
    cfg.service(
        resource("/collages")
            .route(get().to(self::get_subscription_collages::exec::<R>))
            .route(post().to(self::create_subscription_collages::exec::<R>))
            .route(delete().to(self::remove_subscription_collages::exec::<R>)),
    );
    cfg.service(
        resource("/torrent-request-comments")
            .route(get().to(self::get_subscription_torrent_request_comments::exec::<R>))
            .route(post().to(self::create_subscription_torrent_request_comments::exec::<R>))
            .route(delete().to(self::remove_subscription_torrent_request_comments::exec::<R>)),
    );
}
