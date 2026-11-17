use crate::{
    common::{arcadia_with_plugin, spawn_http_server, website},
    mocks::mock_redis::MockRedisPool,
};
use arcadia_api::{
    Arcadia,
    config::ExternalSourcePlugin,
    handlers::scrapers::ScrapedPublicRating,
    services::public_rating_service::{
        create_public_ratings_to_fetch, fetch_all_due_public_ratings, save_scraped_public_rating,
    },
};
use arcadia_storage::{connection_pool::ConnectionPool, models::title_group::ContentType};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::{collections::BTreeMap, sync::Arc};

const TITLE_GROUP_ID: i32 = 1;

/// A plugin collecting the ratings of the links on the given domains, at `plugin_url`.
async fn arcadia_with_rating_plugin(
    pool: Arc<ConnectionPool>,
    domains: &[&str],
    plugin_url: &str,
) -> actix_web::web::Data<Arcadia<MockRedisPool>> {
    let sources = domains
        .iter()
        .map(|domain| {
            let mut website = website(vec![ContentType::Music]);
            website.domain = domain.to_string();
            (domain.to_string(), website)
        })
        .collect::<BTreeMap<_, _>>();
    arcadia_with_plugin(
        pool,
        ExternalSourcePlugin {
            id: String::from("example"),
            placeholder: String::from("Example url"),
            sources,
            title_url: format!("{plugin_url}/scrape"),
            ratings_url: Some(format!("{plugin_url}/ratings")),
            timeout_seconds: 30,
        },
    )
    .await
}

/// Asks a plugin answering `answer`, and accepting the links of the given domains, for the ratings
/// of the links of the title group (on musicbrainz.org and www.discogs.com, neither is a TMDB one).
/// Returns how many ratings were created, the stored ratings and the requests the plugin received.
async fn fetch_ratings_from_plugin(
    pool: Arc<ConnectionPool>,
    domains: &[&str],
    answer: Value,
) -> (u64, Value, Vec<String>) {
    let (plugin_url, requests) = spawn_http_server("200 OK", answer.to_string());
    let arc = arcadia_with_rating_plugin(Arc::clone(&pool), domains, &plugin_url).await;
    let title_group = pool.find_title_group(TITLE_GROUP_ID).await.unwrap();

    let created_ratings_amount = create_public_ratings_to_fetch(&arc, &title_group.external_links)
        .await
        .unwrap();
    assert_eq!(
        fetch_all_due_public_ratings(&arc, None).await.unwrap(),
        created_ratings_amount
    );
    // asked for again only after `public_ratings_retry_seconds`
    assert_eq!(fetch_all_due_public_ratings(&arc, None).await.unwrap(), 0);

    let title_group = pool.find_title_group(TITLE_GROUP_ID).await.unwrap();
    (
        created_ratings_amount,
        title_group.public_ratings,
        requests.try_iter().collect(),
    )
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_the_rating_a_plugin_answers_with_is_stored(pool: PgPool) {
    let (created_ratings_amount, public_ratings, requests) = fetch_ratings_from_plugin(
        Arc::new(ConnectionPool::with_pg_pool(pool)),
        &["musicbrainz.org", "discogs.com"],
        json!({"status": "rated", "rating": 8.2, "votes": 1234}),
    )
    .await;

    // discogs.com also accepts the links of www.discogs.com
    assert_eq!(created_ratings_amount, 2);
    assert_eq!(public_ratings.as_array().unwrap().len(), 2);
    assert_eq!(public_ratings[0]["rating"], 8.2);
    assert_eq!(public_ratings[0]["votes"], 1234);
    // never collected before, so the plugin is not given a date
    assert!(requests.iter().all(|request| {
        request.starts_with("GET /ratings?url=https") && !request.contains("last_collected_at")
    }));
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_a_plugin_is_only_asked_about_the_links_of_its_websites(pool: PgPool) {
    let (created_ratings_amount, _, requests) = fetch_ratings_from_plugin(
        Arc::new(ConnectionPool::with_pg_pool(pool)),
        &["musicbrainz.org", "example.com"],
        json!({"status": "retry_later"}),
    )
    .await;

    assert_eq!(created_ratings_amount, 1);
    assert_eq!(requests.len(), 1);
    assert!(requests[0].contains("musicbrainz.org"));
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_nothing_is_stored_when_a_plugin_retries_later(pool: PgPool) {
    let (_, public_ratings, _) = fetch_ratings_from_plugin(
        Arc::new(ConnectionPool::with_pg_pool(pool)),
        &["musicbrainz.org", "discogs.com"],
        json!({"status": "retry_later"}),
    )
    .await;

    assert_eq!(public_ratings, json!([]));
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_the_ratings_a_plugin_does_not_support_are_deleted(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    fetch_ratings_from_plugin(
        Arc::clone(&pool),
        &["musicbrainz.org", "discogs.com"],
        json!({"status": "unsupported"}),
    )
    .await;

    // a deleted rating is created again, the existing ones are left untouched
    let title_group = pool.find_title_group(TITLE_GROUP_ID).await.unwrap();
    let links = title_group.external_links;
    let providers = vec![String::from("example"); links.len()];
    let created_ratings_amount = pool
        .create_public_ratings_to_fetch(&links, &providers)
        .await
        .unwrap();
    assert_eq!(created_ratings_amount, 2);
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_a_scraped_rating_is_kept_until_a_title_group_lists_its_link(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let arc = arcadia_with_rating_plugin(
        Arc::clone(&pool),
        &["musicbrainz.org", "discogs.com", "example.com"],
        "http://localhost",
    )
    .await;
    let listed_link = pool
        .find_title_group(TITLE_GROUP_ID)
        .await
        .unwrap()
        .external_links[0]
        .clone();
    for external_link in [
        listed_link.clone(),
        String::from("https://example.com/not-uploaded-yet"),
        // not collected by the plugin, so not saved
        String::from("https://unknown.org/release/1"),
    ] {
        let scraped_rating = ScrapedPublicRating {
            external_link,
            rating: 7.5,
            votes: 10,
        };
        save_scraped_public_rating(&arc, "example", &scraped_rating).await;
    }

    let public_ratings = pool
        .find_title_group(TITLE_GROUP_ID)
        .await
        .unwrap()
        .public_ratings;
    assert_eq!(public_ratings.as_array().unwrap().len(), 1);
    assert_eq!(public_ratings[0]["external_link"], listed_link.as_str());

    // kept during the grace period, then deleted as no title group lists its link
    assert_eq!(pool.delete_unlisted_public_ratings(60.0).await.unwrap(), 0);
    assert_eq!(pool.delete_unlisted_public_ratings(-60.0).await.unwrap(), 1);
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_the_ratings_of_a_replaced_provider_are_reassigned(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let links = pool
        .find_title_group(TITLE_GROUP_ID)
        .await
        .unwrap()
        .external_links;
    let removed_providers = vec![String::from("removed_plugin"); links.len()];
    pool.create_public_ratings_to_fetch(&links, &removed_providers)
        .await
        .unwrap();

    let (reassigned_ratings_amount, public_ratings, _) = fetch_ratings_from_plugin(
        Arc::clone(&pool),
        &["musicbrainz.org", "discogs.com"],
        json!({"status": "rated", "rating": 8.2, "votes": 1234}),
    )
    .await;

    assert_eq!(reassigned_ratings_amount, 2);
    assert_eq!(public_ratings.as_array().unwrap().len(), 2);
}
