use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_shared::tracker::models::env::ArcadiaSettingsForTracker;
use arcadia_storage::{
    models::{arcadia_settings::ArcadiaSettings, user::UserPermission},
    redis::RedisPoolInterface,
};
use reqwest::Method;

#[utoipa::path(
    put,
    operation_id = "Update Arcadia settings",
    tag = "Arcadia Settings",
    path = "/api/arcadia-settings",
    security(
        ("http" = ["Bearer"])
    ),
    request_body = ArcadiaSettings,
    responses(
        (status = 200, description = "Successfully updated Arcadia settings", body=ArcadiaSettings),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    settings: Json<ArcadiaSettings>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditArcadiaSettings, req.path())
        .await?;

    let automated_message_fields = [
        settings.automated_message_on_signup.is_some(),
        settings.automated_message_on_signup_sender_id.is_some(),
        settings.automated_message_on_signup_locked.is_some(),
        settings
            .automated_message_on_signup_conversation_name
            .is_some(),
    ];
    let filled_field_count = automated_message_fields.iter().filter(|&&x| x).count();
    if filled_field_count > 0 && filled_field_count < 4 {
        return Err(arcadia_common::error::Error::InvalidArcadiaSettings(
            "All automated message on signup fields (message, sender_id, locked, conversation_name) must be provided if any is set".to_string(),
        ));
    }

    if settings.torrent_bonus_points_cost_min < 0 {
        return Err(arcadia_common::error::Error::InvalidArcadiaSettings(
            "torrent_bonus_points_cost_min must be greater than or equal to 0".to_string(),
        ));
    }

    if settings.torrent_bonus_points_cost_max < 0 {
        return Err(arcadia_common::error::Error::InvalidArcadiaSettings(
            "torrent_bonus_points_cost_max must be greater than or equal to 0".to_string(),
        ));
    }

    if settings.torrent_bonus_points_cost_min > settings.torrent_bonus_points_cost_max {
        return Err(arcadia_common::error::Error::InvalidArcadiaSettings(
            "torrent_bonus_points_cost_min must be less than or equal to torrent_bonus_points_cost_max".to_string(),
        ));
    }

    if let Some(days) = settings.inactive_user_ban_after_days
        && days <= 0
    {
        return Err(arcadia_common::error::Error::InvalidArcadiaSettings(
            "inactive_user_ban_after_days must be greater than 0".to_string(),
        ));
    }

    if settings.invitation_expiration_days <= 0 {
        return Err(arcadia_common::error::Error::InvalidArcadiaSettings(
            "invitation_expiration_days must be greater than 0".to_string(),
        ));
    }

    if settings.reseed_requestable_after_hours < 1 {
        return Err(arcadia_common::error::Error::InvalidArcadiaSettings(
            "reseed_requestable_after_hours must be at least 1".to_string(),
        ));
    }

    if settings.global_snatch_bonus_points_cost_factor < 0 {
        return Err(arcadia_common::error::Error::InvalidArcadiaSettings(
            "global_snatch_bonus_points_cost_factor must be greater than or equal to 0".to_string(),
        ));
    }

    if settings.global_snatch_bonus_points_reward_factor < 0 {
        return Err(arcadia_common::error::Error::InvalidArcadiaSettings(
            "global_snatch_bonus_points_reward_factor must be greater than or equal to 0"
                .to_string(),
        ));
    }

    let updated_settings = arc.pool.update_arcadia_settings(&settings).await?;

    // Update the in-memory settings
    *arc.settings.lock().unwrap() = updated_settings.clone();

    // Notify tracker of settings change
    let payload = ArcadiaSettingsForTracker {
        global_upload_factor: updated_settings.global_upload_factor,
        global_download_factor: updated_settings.global_download_factor,
        global_snatch_bonus_points_cost_factor: updated_settings
            .global_snatch_bonus_points_cost_factor,
        global_snatch_bonus_points_reward_factor: updated_settings
            .global_snatch_bonus_points_reward_factor,
        snatched_torrent_bonus_points_transferred_to: updated_settings
            .snatched_torrent_bonus_points_transferred_to
            .clone(),
        charge_bonus_points_on_resnatch: updated_settings.charge_bonus_points_on_resnatch,
    };

    arc.pool
        .notify_tracker(
            Method::PUT,
            &["settings"],
            Some(&payload),
            "update settings",
        )
        .await;

    Ok(HttpResponse::Ok().json(updated_settings))
}
