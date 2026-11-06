use crate::{
    handlers::scrapers::ExternalDBData,
    middlewares::auth_middleware::Authdata,
    services::external_db_service::{
        check_if_existing_title_group_with_link_exists, respond_with_scraped_data,
    },
    Arcadia,
};
use actix_web::{web::Data, HttpResponse};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        edition_group::{create_default_edition_group, UserCreatedEditionGroup},
        title_group::{create_default_title_group, ContentType, UserCreatedTitleGroup},
    },
    redis::RedisPoolInterface,
};
use musicbrainz_rs::{
    api_bindium::{
        ureq::{config::Config, Agent, Proxy},
        ApiClient,
    },
    client::MusicBrainzClient,
    entity::{
        release::Release,
        release_group::{ReleaseGroup, ReleaseGroupPrimaryType},
        CoverartResponse,
    },
    Fetch, FetchCoverart,
};
use regex::Regex;
use serde_json::json;

async fn get_musicbrainz_release_group_data(
    id: &str,
    client: &MusicBrainzClient,
) -> Result<UserCreatedTitleGroup> {
    // TODO: handle artists
    let musicbrainz_title_group = ReleaseGroup::fetch()
        .id(id)
        .with_tags()
        .with_aliases()
        // .with_artists()
        .execute_with_client_async(client)
        .await
        .map_err(|e| Error::ErrorGettingMusicbrainzData(Box::new(e)))?;
    let cover = ReleaseGroup::fetch_coverart()
        .id(id)
        .execute_with_client_async(client)
        .await
        .map_or_else(
            |_| String::new(),
            |c| match c {
                CoverartResponse::Url(url) => url,
                CoverartResponse::Json(coverart) => coverart
                    .images
                    .first()
                    .map_or_else(String::new, |img| img.image.clone()),
            },
        );

    Ok(UserCreatedTitleGroup {
        name: musicbrainz_title_group.title,
        name_aliases: musicbrainz_title_group
            .aliases
            .unwrap()
            .iter()
            .map(|alias| alias.name.clone())
            .collect(),
        tags: musicbrainz_title_group
            .tags
            .unwrap()
            .iter()
            .map(|tag| tag.name.clone().replace(" ", "."))
            .collect(),
        original_release_date: musicbrainz_title_group
            .first_release_date
            .and_then(|date_string| date_string.into_naive_date(1, 1, 1).ok()),
        category: Some(
            musicbrainz_title_group
                .primary_type
                .unwrap_or(ReleaseGroupPrimaryType::UnrecognizedReleaseGroupPrimaryType)
                .into(),
        ),
        content_type: ContentType::Music,
        external_links: vec![format!("https://musicbrainz.org/release-group/{}", id)],
        covers: vec![cover],
        ..create_default_title_group()
    })
}

async fn get_musicbrainz_release_data(
    id: &str,
    client: &MusicBrainzClient,
) -> Result<(UserCreatedEditionGroup, Option<String>)> {
    let musicbrainz_edition_group = Release::fetch()
        .id(id)
        .with_release_groups()
        .with_labels()
        .execute_with_client_async(client)
        .await
        .map_err(|e| Error::ErrorGettingMusicbrainzData(Box::new(e)))?;
    Ok((
        UserCreatedEditionGroup {
            additional_information: Some(json!({
                "catalogue_number":  musicbrainz_edition_group.label_info.as_ref().and_then(|li| li.first()).and_then(|li_item| li_item.catalog_number.clone()).unwrap_or_default(), //musicbrainz_edition_group.barcode.clone().unwrap_or("".to_string()),
                "label": musicbrainz_edition_group.label_info.as_ref().and_then(|li| li.first()).and_then(|li_item| li_item.label.as_ref()).map(|label| label.name.clone()).unwrap_or_default()
            })),
            release_date: musicbrainz_edition_group
                .date
                .and_then(|date_string| date_string.into_naive_date(1, 1, 1).ok()),
            external_links: vec![format!("https://musicbrainz.org/release/{}", id)],
            ..create_default_edition_group()
        },
        match musicbrainz_edition_group.release_group {
            Some(release_group) => Some(release_group.id),
            _ => None,
        },
    ))
}

#[derive(Debug, PartialEq)]
pub enum MusicBrainzResourceType {
    Release,
    ReleaseGroup,
}

pub async fn exec<R: RedisPoolInterface + 'static>(
    url: &str,
    arc: &Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let (entity_type, id) = Regex::new(r"musicbrainz.org/(release|release-group)/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})")
        .expect("Regex error")
        .captures(url).map(|caps| (match caps[1].as_ref() { "release" => MusicBrainzResourceType::Release, _ => MusicBrainzResourceType::ReleaseGroup }, caps[2].to_string()))
        .ok_or_else(|| Error::InvalidMusicbrainzUrl)?;
    // .expect("No MusicBrainz release/release-group match found in URL");
    // musicbrainz_rs uses its own ureq agent instead of reqwest, so the proxy is set on it directly
    let agent_config = Config::builder()
        .user_agent(format!("{} ({})", arc.tracker.name, arc.api.frontend_url))
        .proxy(arc.api.http_proxy.as_deref().map(|proxy_url| {
            Proxy::new(proxy_url).expect("HTTP_PROXY contains an invalid proxy URL")
        }))
        .build();
    let client = MusicBrainzClient::builder()
        .api_client(
            ApiClient::builder()
                .agent(Agent::new_with_config(agent_config))
                .build(),
        )
        .build();

    let mut title_group: Option<UserCreatedTitleGroup> = None;
    let mut edition_group: Option<UserCreatedEditionGroup> = None;
    match entity_type {
        MusicBrainzResourceType::ReleaseGroup => {
            if let Some(response) =
                check_if_existing_title_group_with_link_exists(&arc.pool, url).await?
            {
                return Ok(response);
            }
            let tg = get_musicbrainz_release_group_data(&id, &client).await?;
            title_group = Some(tg);
        }
        MusicBrainzResourceType::Release => {
            let (eg, release_group_id) = get_musicbrainz_release_data(&id, &client).await?;
            edition_group = Some(eg);
            if let Some(rgid) = release_group_id {
                if let Some(response) = check_if_existing_title_group_with_link_exists(
                    &arc.pool,
                    &format!("https://musicbrainz.org/release-group/{rgid}"),
                )
                .await?
                {
                    return Ok(response);
                }
                let tg = get_musicbrainz_release_group_data(&rgid, &client).await?;
                title_group = Some(tg);
            }
        }
    };

    let external_db_data = ExternalDBData {
        title_group,
        edition_group,
        affiliated_artists: vec![],
        existing_title_group_id: None,
    };

    respond_with_scraped_data(
        &arc.pool,
        arc.rehost_image_host(),
        user.sub,
        external_db_data,
    )
    .await
}
