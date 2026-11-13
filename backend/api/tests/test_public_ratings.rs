use crate::common::{arcadia_with_plugin, spawn_http_server, website};
use arcadia_api::{
    config::ExternalSourcePlugin,
    services::public_rating_service::{
        create_public_ratings_to_fetch, fetch_all_due_public_ratings,
    },
};
use arcadia_storage::{connection_pool::ConnectionPool, models::title_group::ContentType};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::{collections::BTreeMap, sync::Arc};

const TITLE_GROUP_ID: i32 = 1;

/// Asks a plugin answering `answer`, and accepting the links of the given domains, for the ratings
/// of the links of the title group (on musicbrainz.org and www.discogs.com, neither is a TMDB one).
/// Returns how many ratings were created, the stored ratings and the requests the plugin received.
async fn fetch_ratings_from_plugin(
    pool: Arc<ConnectionPool>,
    domains: &[&str],
    answer: Value,
) -> (u64, Value, Vec<String>) {
    let (plugin_url, requests) = spawn_http_server("200 OK", answer.to_string());
    let sources = domains
        .iter()
        .map(|domain| {
            let mut website = website(vec![ContentType::Music]);
            website.domain = domain.to_string();
            (domain.to_string(), website)
        })
        .collect::<BTreeMap<_, _>>();
    let arc = arcadia_with_plugin(
        Arc::clone(&pool),
        ExternalSourcePlugin {
            id: String::from("example"),
            placeholder: String::from("Example url"),
            sources,
            title_url: format!("{plugin_url}/scrape"),
            ratings_url: Some(format!("{plugin_url}/ratings")),
            timeout_seconds: 30,
        },
    )
    .await;
    let title_group = pool.find_title_group(TITLE_GROUP_ID).await.unwrap();

    let created_ratings_amount =
        create_public_ratings_to_fetch(&arc, &[(TITLE_GROUP_ID, title_group.external_links)])
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
        .create_public_ratings_to_fetch(&vec![TITLE_GROUP_ID; links.len()], &links, &providers)
        .await
        .unwrap();
    assert_eq!(created_ratings_amount, 2);
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_the_ratings_of_removed_links_are_deleted(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (_, public_ratings, _) = fetch_ratings_from_plugin(
        Arc::clone(&pool),
        &["musicbrainz.org", "discogs.com"],
        json!({"status": "rated", "rating": 4, "votes": 3}),
    )
    .await;
    let kept_link = public_ratings[0]["external_link"]
        .as_str()
        .unwrap()
        .to_string();

    pool.delete_public_ratings_of_removed_links(TITLE_GROUP_ID, std::slice::from_ref(&kept_link))
        .await
        .unwrap();

    let title_group = pool.find_title_group(TITLE_GROUP_ID).await.unwrap();
    let public_ratings = title_group.public_ratings.as_array().unwrap();
    assert_eq!(public_ratings.len(), 1);
    assert_eq!(public_ratings[0]["external_link"], kept_link);
}
