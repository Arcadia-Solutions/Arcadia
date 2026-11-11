pub mod get_notification_counts;
pub mod get_notifications;
pub mod mark_reseed_requests_as_read;
pub mod mark_torrent_deletions_as_read;

use actix_web::web::{ServiceConfig, get, post, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(resource("").route(get().to(self::get_notifications::exec::<R>)))
        .service(resource("/counts").route(get().to(self::get_notification_counts::exec::<R>)))
        .service(
            resource("/torrent-deletions/read")
                .route(post().to(self::mark_torrent_deletions_as_read::exec::<R>)),
        )
        .service(
            resource("/reseed-requests/{torrent_id}/read")
                .route(post().to(self::mark_reseed_requests_as_read::exec::<R>)),
        );
}
