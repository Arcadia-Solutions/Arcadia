use std::time::Duration;

use crate::{
    Arcadia,
    handlers::scrapers::{ExternalDBData, ScrapedExternalData},
    middlewares::auth_middleware::Authdata,
    services::external_db_service::{
        check_if_existing_title_group_with_link_exists, create_affiliated_artists,
        respond_with_scraped_data,
    },
};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{models::title_group::ContentType, redis::RedisPoolInterface};
use serde::{Deserialize, Serialize};

/// What a plugin is asked to scrape. The content type is the one the uploader picked, and is
/// context rather than an order (can be overriden by the scraper's response)
#[derive(Serialize)]
struct PluginQuery<'a> {
    url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<ContentType>,
}

/// What a plugin may answer a failed request with, to explain the failure to the uploader.
#[derive(Deserialize)]
struct PluginFailure {
    error: String,
}

pub async fn exec<R: RedisPoolInterface + 'static>(
    source_id: &str,
    url: &str,
    content_type: Option<ContentType>,
    arc: &Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let plugin = arc
        .scrapers
        .iter()
        .find(|plugin| plugin.id == source_id)
        .ok_or_else(|| Error::ExternalSourceNotFound(source_id.to_string()))?;

    if let Some(response) = check_if_existing_title_group_with_link_exists(&arc.pool, url).await? {
        return Ok(response);
    }

    // a plugin that did not explain itself is only logged, as the failure is of no use to the
    // uploader
    let plugin_error = |error: String| {
        log::warn!(
            "external source plugin '{}' ({}) failed: {error}",
            plugin.id,
            plugin.url
        );
        Error::ExternalSourcePluginError(plugin.id.clone())
    };

    // plugins run alongside arcadia, so their requests must never go through the outbound proxy
    let response = arc
        .internal_http_client
        .get(&plugin.url)
        .query(&PluginQuery { url, content_type })
        .timeout(Duration::from_secs(plugin.timeout_seconds))
        .send()
        .await
        .map_err(|error| plugin_error(error.to_string()))?;

    let status = response.status();
    let body = response
        .bytes()
        .await
        .map_err(|error| plugin_error(error.to_string()))?;

    if !status.is_success() {
        // the message a plugin answers with is written for the uploader, and shown to them as is
        return Err(match serde_json::from_slice::<PluginFailure>(&body) {
            Ok(failure) if !failure.error.trim().is_empty() => {
                Error::ExternalSourcePluginMessage(failure.error)
            }
            _ => plugin_error(format!("answered with {status}")),
        });
    }

    let scraped_data = serde_json::from_slice::<ScrapedExternalData>(&body)
        .map_err(|error| plugin_error(error.to_string()))?;

    let external_db_data = ExternalDBData {
        title_group: scraped_data.title_group,
        edition_group: scraped_data.edition_group,
        affiliated_artists: create_affiliated_artists(
            &arc.pool,
            scraped_data.affiliated_artists,
            user.sub,
        )
        .await?,
        existing_title_group_id: None,
    };

    // the plugin returns the source link, in the title group's external_links itself (in case it needs
    // to clean it on  its own), so it is not added here.
    // The built-in scrapers still add their own.

    respond_with_scraped_data(
        &arc.pool,
        arc.rehost_image_host(),
        user.sub,
        external_db_data,
    )
    .await
}
