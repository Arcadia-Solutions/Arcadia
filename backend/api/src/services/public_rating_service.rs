use std::time::Duration;

use actix_web::web::Data;
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        title_group::ContentType,
        title_group_public_rating::{CollectedPublicRating, PublicRatingToFetch},
    },
    redis::RedisPoolInterface,
};
use chrono::{DateTime, TimeDelta, Utc};
use futures::future::join_all;
use serde::{Deserialize, Serialize};

use crate::{
    Arcadia,
    handlers::external_db::get_tmdb_data::{build_tmdb_client, extract_media_type_and_id},
};

/// The provider of the built in TMDB ratings.
const TMDB_PROVIDER: &str = "tmdb";

/// What a provider answers when asked for the rating of a link.
#[derive(Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum RatingAnswer {
    Rated(CollectedPublicRating),
    /// No new rating for now, e.g. the stored one is recent enough.
    RetryLater,
    /// The provider does not handle the link, its rating is deleted.
    Unsupported,
}

/// What a scraper plugin is asked the rating of.
#[derive(Serialize)]
struct PluginRatingQuery<'a> {
    url: &'a str,
    /// absent when no rating was collected yet
    #[serde(skip_serializing_if = "Option::is_none")]
    last_collected_at: Option<DateTime<Utc>>,
}

/// TMDB when its API key is set, and the scraper plugins collecting ratings.
fn rating_providers<R: RedisPoolInterface>(arc: &Arcadia<R>) -> Vec<String> {
    arc.api
        .tmdb_api_key
        .as_ref()
        .map(|_| TMDB_PROVIDER.to_string())
        .into_iter()
        .chain(
            arc.scrapers
                .iter()
                .filter(|plugin| plugin.ratings_url.is_some())
                .map(|plugin| plugin.id.clone()),
        )
        .collect()
}

/// The providers asked for the rating of the link: TMDB for its own links when its API key is set,
/// and the plugins collecting ratings for the links on the domains of their websites.
fn link_providers<'a, R: RedisPoolInterface>(
    arc: &'a Arcadia<R>,
    link: &'a str,
) -> impl Iterator<Item = &'a str> {
    let tmdb = (arc.api.tmdb_api_key.is_some() && extract_media_type_and_id(link).is_ok())
        .then_some(TMDB_PROVIDER);
    let plugins = arc
        .scrapers
        .iter()
        .filter(move |plugin| plugin.ratings_url.is_some() && plugin.accepts_link(link))
        .map(|plugin| plugin.id.as_str());
    tmdb.into_iter().chain(plugins)
}

/// Creates the ratings to fetch for the links of the given title groups. Returns how many were
/// created.
pub async fn create_public_ratings_to_fetch<R: RedisPoolInterface>(
    arc: &Arcadia<R>,
    title_groups_external_links: &[(i32, Vec<String>)],
) -> Result<u64> {
    let mut title_group_ids = Vec::new();
    let mut external_links = Vec::new();
    let mut providers = Vec::new();
    for (title_group_id, links) in title_groups_external_links {
        for link in links {
            for provider in link_providers(arc, link) {
                title_group_ids.push(*title_group_id);
                external_links.push(link.clone());
                providers.push(provider.to_string());
            }
        }
    }

    arc.pool
        .create_public_ratings_to_fetch(&title_group_ids, &external_links, &providers)
        .await
}

/// Matches the title group's ratings to its links, then fetches the new ones in the background.
/// The title group is saved already, so a failure is only logged: its ratings are created again at
/// its next edit or by the maintenance tool.
pub async fn sync_title_group_public_ratings<R: RedisPoolInterface + 'static>(
    arc: &Data<Arcadia<R>>,
    title_group_id: i32,
    external_links: &[String],
) {
    let result = async {
        arc.pool
            .delete_public_ratings_of_removed_links(title_group_id, external_links)
            .await?;
        create_public_ratings_to_fetch(arc, &[(title_group_id, external_links.to_vec())]).await
    }
    .await;

    match result {
        Ok(_) => spawn_public_ratings_fetch(arc.clone(), Some(title_group_id)),
        Err(error) => log::error!(
            "could not sync the public ratings of title group {title_group_id}: {error}"
        ),
    }
}

/// Fetches every rating due in the background. Restricted to one title group when
/// `title_group_id` is set.
pub fn spawn_public_ratings_fetch<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    title_group_id: Option<i32>,
) {
    tokio::spawn(async move {
        if let Err(error) = fetch_all_due_public_ratings(&arc, title_group_id).await {
            log::error!("could not fetch the public ratings: {error}");
        }
    });
}

/// Asks the providers for every rating that was never asked for, or not for
/// `public_ratings_retry_seconds`, `public_ratings_batch_size` at a time. Restricted to one title
/// group when `title_group_id` is set. Returns how many were asked for.
pub async fn fetch_all_due_public_ratings<R: RedisPoolInterface>(
    arc: &Arcadia<R>,
    title_group_id: Option<i32>,
) -> Result<u64> {
    let providers = rating_providers(arc);
    let mut asked_amount = 0;
    loop {
        // claiming a rating marks it as tried, so it is out of the next batches
        let ratings_to_fetch = arc
            .pool
            .claim_public_ratings_to_fetch(
                title_group_id,
                &providers,
                arc.periodic_tasks.public_ratings_retry_seconds.get() as f64,
                arc.periodic_tasks.public_ratings_batch_size,
            )
            .await?;

        join_all(
            ratings_to_fetch
                .iter()
                .map(|rating_to_fetch| fetch_public_rating(arc, rating_to_fetch)),
        )
        .await
        .into_iter()
        .collect::<Result<()>>()?;

        asked_amount += ratings_to_fetch.len() as u64;
        if ratings_to_fetch.len() < arc.periodic_tasks.public_ratings_batch_size as usize {
            return Ok(asked_amount);
        }
    }
}

/// A provider failing is only logged, the rating is asked for again at the next retry.
async fn fetch_public_rating<R: RedisPoolInterface>(
    arc: &Arcadia<R>,
    rating_to_fetch: &PublicRatingToFetch,
) -> Result<()> {
    let answer = if rating_to_fetch.provider == TMDB_PROVIDER {
        ask_tmdb(arc, rating_to_fetch).await
    } else {
        ask_plugin(arc, rating_to_fetch).await
    };

    match answer {
        Ok(RatingAnswer::Rated(collected_rating)) => {
            arc.pool
                .save_public_rating(rating_to_fetch, &collected_rating)
                .await
        }
        Ok(RatingAnswer::Unsupported) => arc.pool.delete_public_rating(rating_to_fetch).await,
        Ok(RatingAnswer::RetryLater) => Ok(()),
        Err(error) => {
            log::warn!(
                "could not get the rating of '{}' from '{}': {error}",
                rating_to_fetch.external_link,
                rating_to_fetch.provider
            );
            Ok(())
        }
    }
}

async fn ask_plugin<R: RedisPoolInterface>(
    arc: &Arcadia<R>,
    rating_to_fetch: &PublicRatingToFetch,
) -> std::result::Result<RatingAnswer, String> {
    let (ratings_url, timeout_seconds) = arc
        .scrapers
        .iter()
        .find(|plugin| plugin.id == rating_to_fetch.provider)
        .and_then(|plugin| Some((plugin.ratings_url.as_ref()?, plugin.timeout_seconds)))
        .ok_or("the plugin no longer collects ratings")?;

    // plugins run alongside arcadia, so their requests must never go through the outbound proxy
    arc.internal_http_client
        .get(ratings_url)
        .query(&PluginRatingQuery {
            url: &rating_to_fetch.external_link,
            last_collected_at: rating_to_fetch.collected_at,
        })
        .timeout(Duration::from_secs(timeout_seconds))
        .send()
        .await
        .and_then(|response| response.error_for_status())
        .map_err(|error| error.to_string())?
        .json::<RatingAnswer>()
        .await
        .map_err(|error| error.to_string())
}

async fn ask_tmdb<R: RedisPoolInterface>(
    arc: &Arcadia<R>,
    rating_to_fetch: &PublicRatingToFetch,
) -> std::result::Result<RatingAnswer, String> {
    // the rating of a recent release still moves a lot, an older one barely does
    let is_recent_release = rating_to_fetch
        .title_group_original_release_date
        .is_some_and(|release_date| Utc::now().date_naive() - release_date < TimeDelta::days(30));
    let refresh_after = if is_recent_release {
        TimeDelta::days(1)
    } else {
        TimeDelta::days(60)
    };
    if rating_to_fetch
        .collected_at
        .is_some_and(|collected_at| Utc::now() - collected_at < refresh_after)
    {
        return Ok(RatingAnswer::RetryLater);
    }
    let (media_type, id) = extract_media_type_and_id(&rating_to_fetch.external_link)
        .map_err(|error| error.to_string())?;
    let api_key = arc
        .api
        .tmdb_api_key
        .clone()
        .ok_or("the TMDB API key is not set")?;
    let client = build_tmdb_client(&arc.http_client, api_key);

    let (rating, votes) = if media_type == ContentType::Movie {
        let movie = client
            .get_movie_details(id, &Default::default())
            .await
            .map_err(|error| error.to_string())?;
        (movie.inner.vote_average, movie.inner.vote_count)
    } else {
        let tv_show = client
            .get_tvshow_details(id, &Default::default())
            .await
            .map_err(|error| error.to_string())?;
        (tv_show.inner.vote_average, tv_show.inner.vote_count)
    };

    Ok(RatingAnswer::Rated(CollectedPublicRating {
        rating,
        votes: votes as i64,
    }))
}
