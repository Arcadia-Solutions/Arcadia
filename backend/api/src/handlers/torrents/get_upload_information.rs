use actix_web::{web::Data, HttpResponse};
use arcadia_storage::redis::RedisPoolInterface;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    handlers::{external_db::built_in_external_sources, scrapers::ExternalSource},
    middlewares::auth_middleware::Authdata,
    Arcadia,
};
use arcadia_common::{error::Result, services::torrent_service::get_announce_url};

#[derive(Debug, Serialize, ToSchema)]
pub struct UploadInformation {
    announce_url: String,
    upload_page_top_text: Option<String>,
    bonus_points_given_on_upload: i64,
    allow_uploader_set_torrent_bonus_points_cost: bool,
    default_torrent_bonus_points_cost: i64,
    /// External sources available on this instance, built in ones and the ones provided by plugins.
    external_sources: Vec<ExternalSource>,
}

#[utoipa::path(
    get,
    operation_id = "Get upload information",
    tag = "Torrent",
    path = "/api/torrents/upload-info",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Information related to uploading", body=UploadInformation),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let current_user = arc.pool.find_user_with_id(user.sub).await?;
    let announce_url = get_announce_url(&current_user.passkey, arc.tracker.url.as_ref());
    let mut external_sources = built_in_external_sources();
    external_sources.extend(arc.scrapers.iter().map(|plugin| plugin.external_source()));

    let settings = arc.settings.lock().unwrap();

    Ok(HttpResponse::Ok().json(UploadInformation {
        announce_url,
        upload_page_top_text: settings.upload_page_top_text.clone(),
        bonus_points_given_on_upload: settings.bonus_points_given_on_upload,
        allow_uploader_set_torrent_bonus_points_cost: settings
            .allow_uploader_set_torrent_bonus_points_cost,
        default_torrent_bonus_points_cost: settings.default_torrent_bonus_points_cost,
        external_sources,
    }))
}
