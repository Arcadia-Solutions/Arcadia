pub mod create_torrent_request;
pub mod create_torrent_request_comment;
pub mod create_torrent_request_vote;
pub mod delete_torrent_request;
pub mod delete_torrent_request_comment;
pub mod edit_torrent_request;
pub mod fill_torrent_request;
pub mod get_torrent_request;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_torrent_request::exec::<R>))
            .route(get().to(self::get_torrent_request::exec::<R>))
            .route(put().to(self::edit_torrent_request::exec::<R>))
            .route(delete().to(self::delete_torrent_request::exec::<R>)),
    );
    cfg.service(resource("/fill").route(post().to(self::fill_torrent_request::exec::<R>)));
    cfg.service(resource("/vote").route(post().to(self::create_torrent_request_vote::exec::<R>)));
    cfg.service(
        resource("/comment").route(post().to(self::create_torrent_request_comment::exec::<R>)),
    );
    cfg.service(
        resource("/comment/{id}")
            .route(delete().to(self::delete_torrent_request_comment::exec::<R>)),
    );
}
