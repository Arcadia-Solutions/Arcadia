pub mod check_missing_registration_ips;
pub mod recompute_cached_amounts;
pub mod rehash_torrents;

use actix_web::web::{post, resource, ServiceConfig};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(resource("/rehash-torrents").route(post().to(self::rehash_torrents::exec::<R>)));
    cfg.service(
        resource("/recompute-cached-amounts")
            .route(post().to(self::recompute_cached_amounts::exec::<R>)),
    );
    cfg.service(
        resource("/check-missing-registration-ips")
            .route(post().to(self::check_missing_registration_ips::exec::<R>)),
    );
}
