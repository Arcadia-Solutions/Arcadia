use std::{str::FromStr, sync::LazyLock};

use crate::{
    Arcadia,
    handlers::scrapers::{ExternalDBData, ScrapedAffiliatedArtist},
    middlewares::auth_middleware::Authdata,
    services::external_db_service::{
        check_if_existing_title_group_with_link_exists, create_affiliated_artists,
        respond_with_scraped_data,
    },
};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        artist::ArtistRole,
        edition_group::{UserCreatedEditionGroup, create_default_edition_group},
        title_group::{ContentType, UserCreatedTitleGroup, create_default_title_group},
        torrent::Language,
    },
    redis::RedisPoolInterface,
};
use regex::Regex;
use tmdb_api::client::Client;
use tmdb_api::client::reqwest::Client as ReqwestClient;
use tmdb_api::common::credits::{Cast, Crew};

fn map_crew_job_to_role(job: &str) -> Option<ArtistRole> {
    match job {
        "Director" => Some(ArtistRole::Director),
        "Producer" | "Executive Producer" => Some(ArtistRole::Producer),
        "Writer" | "Screenplay" | "Story" => Some(ArtistRole::Writer),
        "Original Music Composer" | "Music" => Some(ArtistRole::Composer),
        "Director of Photography" => Some(ArtistRole::Cinematographer),
        "Editor" => Some(ArtistRole::Editor),
        _ => None,
    }
}

fn map_crew_job_to_roles(job: &str) -> Vec<ArtistRole> {
    map_crew_job_to_role(job).into_iter().collect()
}

fn tmdb_profile_picture_url(profile_path: &Option<String>) -> Vec<String> {
    profile_path
        .as_ref()
        .map(|path| vec![format!("https://image.tmdb.org/t/p/w500{path}")])
        .unwrap_or_default()
}

fn tmdb_person_url(person_id: u64) -> Vec<String> {
    vec![format!("https://www.themoviedb.org/person/{person_id}")]
}

fn scraped_artists_from_credits(cast: &[Cast], crew: &[Crew]) -> Vec<ScrapedAffiliatedArtist> {
    let mut scraped_artists: Vec<ScrapedAffiliatedArtist> = Vec::new();

    for member in cast {
        scraped_artists.push(ScrapedAffiliatedArtist {
            name: member.person.name.clone(),
            aliases: vec![],
            description: String::new(),
            pictures: tmdb_profile_picture_url(&member.person.profile_path),
            external_links: tmdb_person_url(member.person.id),
            roles: vec![ArtistRole::Actor],
            nickname: Some(member.character.clone()),
        });
    }

    for member in crew {
        scraped_artists.push(ScrapedAffiliatedArtist {
            name: member.person.name.clone(),
            aliases: vec![],
            description: String::new(),
            pictures: tmdb_profile_picture_url(&member.person.profile_path),
            external_links: tmdb_person_url(member.person.id),
            roles: map_crew_job_to_roles(&member.job),
            nickname: None,
        });
    }

    scraped_artists
}

async fn get_tmdb_movie_data(client: &Client<ReqwestClient>, id: u64) -> Result<ExternalDBData> {
    let tmdb_movie = client
        .get_movie_details(id, &Default::default())
        .await
        .unwrap();
    let mut title_group = UserCreatedTitleGroup {
        name: tmdb_movie.inner.original_title.clone(),
        name_aliases: (tmdb_movie.inner.title != tmdb_movie.inner.original_title)
            .then_some(vec![tmdb_movie.inner.original_title])
            .unwrap_or_default(),
        tags: tmdb_movie
            .genres
            .iter()
            .map(|g| g.name.clone().to_lowercase())
            .collect(),
        description: tmdb_movie.inner.overview,
        original_language: Some(
            Language::from_str(&tmdb_movie.inner.original_language).unwrap_or(Language::Other),
        ),
        original_release_date: tmdb_movie.inner.release_date,
        covers: vec![
            tmdb_movie
                .inner
                .poster_path
                .map(|path| format!("https://image.tmdb.org/t/p/w1280{path}"))
                .unwrap_or("".to_string()),
        ],
        content_type: ContentType::Movie,
        ..create_default_title_group()
    };

    if let Some(link) = tmdb_movie
        .imdb_id
        .map(|id| format!("https://www.imdb.com/title/{id}"))
    {
        title_group.external_links = vec![link];
    }

    let edition_group = UserCreatedEditionGroup {
        release_date: title_group.original_release_date,
        ..create_default_edition_group()
    };
    Ok(ExternalDBData {
        title_group: Some(title_group),
        edition_group: Some(edition_group),
        affiliated_artists: vec![],
        existing_title_group_id: None,
    })
}

pub async fn exec<R: RedisPoolInterface + 'static>(
    url: &str,
    arc: &Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    if let Some(response) = check_if_existing_title_group_with_link_exists(&arc.pool, url).await? {
        return Ok(response);
    }

    if arc.api.tmdb_api_key.is_none() {
        return Err(Error::TMDBDataFetchingNotAvailable);
    }
    let (media_type, id) = extract_media_type_and_id(url)?;

    let client = build_tmdb_client(&arc.http_client, arc.api.tmdb_api_key.clone().unwrap());

    let mut external_db_data = match media_type {
        ContentType::Movie => get_tmdb_movie_data(&client, id).await?,
        ContentType::TVShow => todo!(),
        // should never happen
        _ => return Err(Error::InvalidTMDBUrl),
    };

    // Fetch credits and create artists
    let credits = match media_type {
        ContentType::Movie => client
            .get_movie_credits(id, &Default::default())
            .await
            .map_err(|_| Error::TMDBDataFetchingError)?,
        _ => unreachable!(),
    };

    external_db_data.affiliated_artists = create_affiliated_artists(
        &arc.pool,
        scraped_artists_from_credits(&credits.cast, &credits.crew),
        user.sub,
    )
    .await?;

    if let Some(title_group) = &mut external_db_data.title_group {
        title_group.external_links.push(url.to_string());
    }

    respond_with_scraped_data(
        &arc.pool,
        arc.rehost_image_host(),
        user.sub,
        external_db_data,
    )
    .await
}

pub fn build_tmdb_client(http_client: &reqwest::Client, api_key: String) -> Client<ReqwestClient> {
    Client::builder()
        .with_executor(http_client.clone())
        .with_api_key(api_key)
        .build()
        .expect("Failed to build TMDB client")
}

pub fn extract_media_type_and_id(tmdb_url: &str) -> Result<(ContentType, u64)> {
    static TMDB_URL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"themoviedb\.org/(movie|tv)/(\d+)(?:-|$)").unwrap());
    let captures = TMDB_URL.captures(tmdb_url).ok_or(Error::InvalidTMDBUrl)?;

    let media_type = match &captures[1] {
        "movie" => ContentType::Movie,
        _ => ContentType::TVShow,
    };
    let id = captures[2]
        .parse::<u64>()
        .map_err(|_| Error::InvalidTMDBUrl)?;

    Ok((media_type, id))
}
