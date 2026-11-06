use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use actix_web::HttpResponse;
use arcadia_common::error::Result;
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::artist::{AffiliatedArtistHierarchy, UserCreatedArtist},
};
use chrono::Utc;

use crate::{
    handlers::scrapers::{ExternalDBData, ScrapedAffiliatedArtist},
    services::image_host_service::{ImageHost, Uploader, rehost_image_urls},
};

/// The treatment every scraper answer goes through, so that no scraper has to
/// remember it: the tags are put in the form arcadia stores them (sanitized,
/// replaced by the tag they are a synonym of, without the deleted ones), and
/// the images are rehosted to `rehost_image_host` when set.
pub async fn respond_with_scraped_data(
    pool: &Arc<ConnectionPool>,
    rehost_image_host: Option<&ImageHost>,
    current_user_id: i32,
    mut external_db_data: ExternalDBData,
) -> Result<HttpResponse> {
    if let Some(title_group) = &mut external_db_data.title_group {
        title_group.tags = pool.resolve_tag_names(&title_group.tags).await?;
    }

    if let Some(image_host) = rehost_image_host {
        rehost_external_db_images(image_host, pool, current_user_id, &mut external_db_data).await;
    }

    Ok(HttpResponse::Ok().json(external_db_data))
}

pub async fn check_if_existing_title_group_with_link_exists(
    pool: &ConnectionPool,
    url: &str,
) -> Result<Option<HttpResponse>> {
    let existing_title_group_id = pool.does_title_group_with_link_exist(url).await?;
    if existing_title_group_id.is_some() {
        return Ok(Some(HttpResponse::Ok().json(ExternalDBData {
            title_group: None,
            edition_group: None,
            affiliated_artists: vec![],
            existing_title_group_id,
        })));
    }
    Ok(None)
}

/// Creates the artists returned by an external source and turns them into affiliated artists.
/// An artist appearing several times (for instance as an actor and as a director) is merged into a
/// single affiliation holding all of its roles.
pub async fn create_affiliated_artists(
    pool: &ConnectionPool,
    scraped_artists: Vec<ScrapedAffiliatedArtist>,
    current_user_id: i32,
) -> Result<Vec<AffiliatedArtistHierarchy>> {
    if scraped_artists.is_empty() {
        return Ok(vec![]);
    }

    // an artist appearing several times is only created once
    let artists_to_create: Vec<UserCreatedArtist> = {
        let mut names_to_create: HashSet<&str> = HashSet::new();
        scraped_artists
            .iter()
            .filter(|scraped_artist| names_to_create.insert(scraped_artist.name.as_str()))
            .map(|scraped_artist| UserCreatedArtist {
                name: scraped_artist.name.clone(),
                aliases: scraped_artist.aliases.clone(),
                description: scraped_artist.description.clone(),
                pictures: scraped_artist.pictures.clone(),
                external_links: scraped_artist.external_links.clone(),
            })
            .collect()
    };

    let created_artists = pool
        .create_artists(&artists_to_create, current_user_id)
        .await?;

    let created_artists_by_name: HashMap<&str, _> = created_artists
        .iter()
        .map(|created_artist| (created_artist.name.as_str(), created_artist))
        .collect();

    let mut affiliated_artists: Vec<AffiliatedArtistHierarchy> = Vec::new();

    for scraped_artist in scraped_artists {
        let Some(artist) = created_artists_by_name
            .get(scraped_artist.name.as_str())
            .copied()
        else {
            continue;
        };

        if let Some(existing_affiliation) = affiliated_artists
            .iter_mut()
            .find(|affiliation| affiliation.artist_id == artist.id)
        {
            for role in scraped_artist.roles {
                if !existing_affiliation.roles.contains(&role) {
                    existing_affiliation.roles.push(role);
                }
            }
            if existing_affiliation.nickname.is_none() {
                existing_affiliation.nickname = scraped_artist.nickname;
            }
        } else {
            affiliated_artists.push(AffiliatedArtistHierarchy {
                id: 0,
                title_group_id: 0,
                artist_id: artist.id,
                roles: scraped_artist.roles,
                nickname: scraped_artist.nickname,
                created_at: Utc::now(),
                created_by_id: current_user_id,
                artist: artist.clone(),
            });
        }
    }

    Ok(affiliated_artists)
}

/// Rehosts the covers of the scraped title group, and the pictures of the scraped artists in the
/// background to avoid blocking the response. They are uploaded on behalf of the current user.
async fn rehost_external_db_images(
    image_host: &ImageHost,
    pool: &Arc<ConnectionPool>,
    current_user_id: i32,
    external_db_data: &mut ExternalDBData,
) {
    let artists: Vec<(i64, Vec<String>)> = external_db_data
        .affiliated_artists
        .iter()
        .filter(|affiliation| !affiliation.artist.pictures.is_empty())
        .map(|affiliation| (affiliation.artist.id, affiliation.artist.pictures.clone()))
        .collect();
    let covers = external_db_data
        .title_group
        .as_mut()
        .map(|title_group| &mut title_group.covers)
        .filter(|covers| !covers.is_empty());
    if covers.is_none() && artists.is_empty() {
        return;
    }

    let uploader = match pool.find_user_with_id(current_user_id).await {
        Ok(user) => Uploader {
            id: user.id,
            username: user.username,
        },
        Err(error) => {
            log::warn!("Failed to find the user to rehost images for: {error}");
            return;
        }
    };

    if let Some(covers) = covers {
        rehost_image_urls(image_host, &uploader, covers).await;
    }

    let image_host = image_host.clone();
    let pool = Arc::clone(pool);
    tokio::spawn(async move {
        for (artist_id, mut pictures) in artists {
            rehost_image_urls(&image_host, &uploader, &mut pictures).await;
            if let Err(error) = pool.update_artist_pictures(artist_id, &pictures).await {
                log::warn!("Failed to update rehosted pictures for artist {artist_id}: {error}");
            }
        }
    });
}
