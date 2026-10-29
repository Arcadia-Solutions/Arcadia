use crate::Arcadia;
use actix_web::{web::Data, HttpResponse};
use arcadia_common::error::Result;
use arcadia_storage::{models::arcadia_settings::PublicArcadiaSettings, redis::RedisPoolInterface};
use chrono::Utc;

#[utoipa::path(
    get,
    operation_id = "Get public Arcadia settings",
    tag = "Arcadia Settings",
    path = "/api/arcadia-settings/public",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved Arcadia public settings", body=PublicArcadiaSettings),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(arc: Data<Arcadia<R>>) -> Result<HttpResponse> {
    let settings = arc.settings.lock().unwrap().clone();
    let running_advent_calendar_button_text = arc
        .pool
        .find_running_advent_calendar_button_text(Utc::now())
        .await?;
    Ok(HttpResponse::Ok().json(PublicArcadiaSettings {
        global_download_factor: settings.global_download_factor,
        global_upload_factor: settings.global_upload_factor,
        open_signups: settings.open_signups,
        logo_subtitle: settings.logo_subtitle,
        bonus_points_alias: settings.bonus_points_alias,
        bonus_points_decimal_places: settings.bonus_points_decimal_places,
        displayed_top_bar_stats: settings.displayed_top_bar_stats,
        displayable_user_stats: settings.displayable_user_stats,
        torrent_request_vote_currencies: settings.torrent_request_vote_currencies,
        available_shop_items: settings.available_shop_items,
        emails_enabled: arc.smtp.is_enabled(),
        irc_enabled: arc.ergo.is_enabled(),
        display_image_host_drag_and_drop: settings.display_image_host_drag_and_drop,
        irc_webchat_enabled: settings.irc_webchat_enabled && arc.ergo.is_enabled(),
        irc_webchat_default_channels: settings.irc_webchat_default_channels.clone(),
        min_amount_tags_title_group: settings.min_amount_tags_title_group,
        reseed_requestable_after_hours: settings.reseed_requestable_after_hours,
        custom_js_code: settings.custom_js_code,
        custom_footer: settings.custom_footer,
        uploads_disabled: settings.uploads_disabled,
        uploads_disabled_message: settings.uploads_disabled_message,
        running_advent_calendar_button_text,
    }))
}
