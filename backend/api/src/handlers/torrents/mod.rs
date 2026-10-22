pub mod create_torrent;
pub mod create_torrent_report;
pub mod delete_torrent;
pub mod delete_torrent_report;
pub mod download_dottorrent_file;
pub mod download_torrents_archive;
pub mod edit_torrent;
pub mod edit_torrent_up_down_factors;
pub mod get_top_torrents;
pub mod get_torrent_activities;
pub mod get_torrent_peers;
pub mod get_torrent_title_group;
pub mod get_upload_information;
pub mod move_torrent_to_edition_group;
pub mod request_reseed;
pub mod set_torrent_staff_checked;

use actix_web::web::{delete, get, post, put, resource, ServiceConfig};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_torrent::exec::<R>))
            .route(get().to(self::download_dottorrent_file::exec::<R>))
            .route(put().to(self::edit_torrent::exec::<R>))
            .route(delete().to(self::delete_torrent::exec::<R>)),
    );
    cfg.service(resource("/{id}/reseed-request").route(post().to(self::request_reseed::exec::<R>)));
    cfg.service(resource("/upload-info").route(get().to(self::get_upload_information::exec::<R>)));
    cfg.service(resource("/top").route(get().to(self::get_top_torrents::exec::<R>)));
    cfg.service(resource("/peers").route(get().to(self::get_torrent_peers::exec::<R>)));
    cfg.service(resource("/activities").route(get().to(self::get_torrent_activities::exec::<R>)));
    cfg.service(
        resource("/title-group-id").route(get().to(self::get_torrent_title_group::exec::<R>)),
    );
    cfg.service(
        resource("/reports")
            .route(post().to(self::create_torrent_report::exec::<R>))
            .route(delete().to(self::delete_torrent_report::exec::<R>)),
    );
    cfg.service(
        resource("/staff-checked").route(put().to(self::set_torrent_staff_checked::exec::<R>)),
    );
    cfg.service(
        resource("/up-down-factors").route(put().to(self::edit_torrent_up_down_factors::exec::<R>)),
    );
    cfg.service(
        resource("/move-to-edition-group")
            .route(put().to(self::move_torrent_to_edition_group::exec::<R>)),
    );
}
