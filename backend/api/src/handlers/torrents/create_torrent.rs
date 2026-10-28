use actix_multipart::form::MultipartForm;
use actix_web::{web::Data, HttpRequest, HttpResponse};
use arcadia_shared::tracker::models::torrent::APIInsertTorrent;
use reqwest::Method;

use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        torrent::{Torrent, UploadedTorrent},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create torrent",
    tag = "Torrent",
    path = "/api/torrents",
    request_body(content = UploadedTorrent, content_type = "multipart/form-data"),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully uploaded the torrent", body=Torrent),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: MultipartForm<UploadedTorrent>,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
    user: Authdata,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::UploadTorrent, req.path())
        .await?;

    if arc.settings.lock().unwrap().uploads_disabled {
        return Err(arcadia_common::error::Error::UploadsDisabled);
    }

    let upload_method = req
        .headers()
        .get("X-Upload-Method")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("manual")
        .to_string();

    let (bonus_points_given_on_upload, bonus_points_snatch_cost, torrent_max_release_date_allowed) = {
        let settings = arc.settings.lock().unwrap();
        let cost = if settings.allow_uploader_set_torrent_bonus_points_cost {
            let user_cost = form.bonus_points_snatch_cost.0;
            if user_cost < settings.torrent_bonus_points_cost_min {
                return Err(
                    arcadia_common::error::Error::BonusPointsSnatchCostOutOfRange(format!(
                        "bonus_points_snatch_cost must be at least {}",
                        settings.torrent_bonus_points_cost_min
                    )),
                );
            }
            if user_cost > settings.torrent_bonus_points_cost_max {
                return Err(
                    arcadia_common::error::Error::BonusPointsSnatchCostOutOfRange(format!(
                        "bonus_points_snatch_cost must be at most {}",
                        settings.torrent_bonus_points_cost_max
                    )),
                );
            }
            user_cost
        } else {
            settings.default_torrent_bonus_points_cost
        };
        (
            settings.bonus_points_given_on_upload,
            cost,
            settings.torrent_max_release_date_allowed,
        )
    };

    let torrent = arc
        .pool
        .create_torrent(
            &form,
            user.sub,
            &upload_method,
            bonus_points_given_on_upload,
            bonus_points_snatch_cost,
            torrent_max_release_date_allowed,
            arc.api.max_torrent_size_bytes,
            arc.tracker.torrent_source_tag.as_deref(),
            &arc.notification_sender,
        )
        .await?;

    let payload = APIInsertTorrent {
        id: torrent.id as u32,
        info_hash: torrent.info_hash,
        is_deleted: false,
        seeders: 0,
        leechers: 0,
        times_completed: 0,
        download_factor: torrent.upload_factor as u8,
        upload_factor: torrent.download_factor as u8,
    };

    arc.pool
        .notify_tracker(
            Method::PUT,
            &["torrents"],
            Some(&payload),
            "insert new torrent",
        )
        .await;

    Ok(HttpResponse::Created().json(torrent))
}
