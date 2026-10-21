pub mod change_user_class;
pub mod change_user_password;
pub mod check_user_registration_ip;
pub mod create_api_key;
pub mod create_irc_account;
pub mod create_password_reset_token;
pub mod create_user_staff_note;
pub mod delete_api_key;
pub mod delete_user_staff_note;
pub mod download_user_torrents;
pub mod edit_user;
pub mod edit_user_permissions;
pub mod edit_user_staff_note;
pub mod get_api_keys;
pub mod get_me;
pub mod get_user;
pub mod get_user_permissions;
pub mod get_user_registration_ip;
pub mod get_user_settings;
pub mod get_user_staff_notes;
pub mod get_user_torrent_activities;
pub mod get_user_torrent_activities_overview;
pub mod lock_user_class;
pub mod remove_user_warnings;
pub mod reset_irc_password;
pub mod search_bonus_points_logs;
pub mod set_user_custom_title;
pub mod set_user_registration_ip_check;
pub mod update_uploaded_torrents_anonymity;
pub mod update_user_settings;
pub mod warn_user;

use actix_web::web::{delete, get, post, put, resource, ServiceConfig};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::get_user::exec::<R>))
            .route(put().to(self::edit_user::exec::<R>)),
    );
    cfg.service(resource("/warn").route(post().to(self::warn_user::exec::<R>)));
    cfg.service(resource("/me").route(get().to(self::get_me::exec::<R>)));
    cfg.service(
        resource("/me/torrents-archive").route(get().to(self::download_user_torrents::exec::<R>)),
    );
    cfg.service(
        resource("/torrent-activities/overview")
            .route(get().to(self::get_user_torrent_activities_overview::exec::<R>)),
    );
    cfg.service(
        resource("/torrent-activities")
            .route(get().to(self::get_user_torrent_activities::exec::<R>)),
    );
    cfg.service(
        resource("/bonus-points-logs").route(get().to(self::search_bonus_points_logs::exec::<R>)),
    );
    cfg.service(
        resource("/api-keys")
            .route(get().to(self::get_api_keys::exec::<R>))
            .route(post().to(self::create_api_key::exec::<R>)),
    );
    cfg.service(resource("/api-keys/{id}").route(delete().to(self::delete_api_key::exec::<R>)));
    cfg.service(
        resource("/irc")
            .route(post().to(self::create_irc_account::exec::<R>))
            .route(put().to(self::reset_irc_password::exec::<R>)),
    );
    cfg.service(
        resource("/settings")
            .route(get().to(self::get_user_settings::exec::<R>))
            .route(put().to(self::update_user_settings::exec::<R>)),
    );
    cfg.service(resource("/password").route(put().to(self::change_user_password::exec::<R>)));
    cfg.service(
        resource("/uploaded-torrents-anonymity")
            .route(put().to(self::update_uploaded_torrents_anonymity::exec::<R>)),
    );
    cfg.service(
        resource("/{id}/permissions")
            .route(get().to(self::get_user_permissions::exec::<R>))
            .route(put().to(self::edit_user_permissions::exec::<R>)),
    );
    cfg.service(resource("/{id}/lock-class").route(put().to(self::lock_user_class::exec::<R>)));
    cfg.service(resource("/{id}/class").route(put().to(self::change_user_class::exec::<R>)));
    cfg.service(
        resource("/{id}/password-reset-token")
            .route(post().to(self::create_password_reset_token::exec::<R>)),
    );
    cfg.service(
        resource("/{id}/custom-title").route(put().to(self::set_user_custom_title::exec::<R>)),
    );
    cfg.service(
        resource("/{id}/warnings").route(delete().to(self::remove_user_warnings::exec::<R>)),
    );
    cfg.service(
        resource("/{id}/registration-ip")
            .route(get().to(self::get_user_registration_ip::exec::<R>))
            .route(put().to(self::set_user_registration_ip_check::exec::<R>)),
    );
    cfg.service(
        resource("/{id}/registration-ip/check")
            .route(get().to(self::check_user_registration_ip::exec::<R>)),
    );
    cfg.service(
        resource("/{id}/staff-notes")
            .route(get().to(self::get_user_staff_notes::exec::<R>))
            .route(post().to(self::create_user_staff_note::exec::<R>)),
    );
    cfg.service(
        resource("/{id}/staff-notes/{staff_note_id}")
            .route(put().to(self::edit_user_staff_note::exec::<R>))
            .route(delete().to(self::delete_user_staff_note::exec::<R>)),
    );
}
