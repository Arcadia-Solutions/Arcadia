pub mod common;
pub mod mocks;

use actix_http::Request;
use actix_web::{
    dev::{Service, ServiceResponse},
    http::StatusCode,
    test, web, App, Error,
};
use arcadia_api::{
    config::{Config, ExternalSourcePlugin},
    Arcadia,
};
use arcadia_storage::{connection_pool::ConnectionPool, models::title_group::ContentType};
use common::{
    auth_header, call_and_read_body_json, call_and_read_body_json_with_status,
    create_test_app_and_login, login_as, spawn_http_server, TestUser,
};
use mocks::mock_redis::MockRedisPool;
use serde::Deserialize;
use sqlx::PgPool;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Deserialize)]
struct ExternalSourceResponse {
    id: String,
    sources: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct UploadInformation {
    external_sources: Vec<ExternalSourceResponse>,
}

#[derive(Deserialize)]
struct ScrapedTitleGroup {
    tags: Vec<String>,
    external_links: Vec<String>,
}

#[derive(Deserialize)]
struct ScrapedArtist {
    name: String,
}

#[derive(Deserialize)]
struct ScrapedAffiliatedArtist {
    roles: Vec<String>,
    nickname: Option<String>,
    artist: ScrapedArtist,
}

#[derive(Deserialize)]
struct ExternalDBData {
    title_group: ScrapedTitleGroup,
    affiliated_artists: Vec<ScrapedAffiliatedArtist>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

/// The plugins are loaded from the configuration file at startup, tests declare them by hand instead.
async fn create_test_app_with_plugin(
    pool: Arc<ConnectionPool>,
    plugin: ExternalSourcePlugin,
) -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    let mut config = arcadia_shared::config::load::<Config>();
    config.scrapers = vec![plugin];
    let settings = pool
        .get_arcadia_settings()
        .await
        .expect("failed to load arcadia settings from database");

    let arc =
        Arcadia::<MockRedisPool>::new(pool, Arc::new(MockRedisPool::default()), config, settings);

    test::init_service(
        App::new()
            .app_data(web::Data::new(arc))
            .configure(arcadia_api::routes::init::<MockRedisPool>),
    )
    .await
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_upload_information_lists_the_built_in_external_sources(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrents/upload-info")
        .to_request();

    let upload_information = call_and_read_body_json::<UploadInformation, _>(&service, req).await;

    let ids: Vec<&str> = upload_information
        .external_sources
        .iter()
        .map(|external_source| external_source.id.as_str())
        .collect();

    // the sources declared in the `scrapers` section, if any, are appended after the built in ones
    assert!(ids.starts_with(&["isbn", "tmdb", "comic-vine", "musicbrainz"]));
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_getting_data_from_an_unknown_external_source_returns_not_found(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/external-sources/unknown?url=https://example.com/something")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_tag_synonyms"),
    migrations = "../storage/migrations"
)]
async fn test_getting_data_from_a_plugin(pool: PgPool) {
    let scraped_data = serde_json::json!({
        "title_group": {
            "name": "Some Movie",
            "name_aliases": [],
            "description": "A movie",
            "original_language": null,
            "country_from": null,
            "covers": [],
            "external_links": ["https://example.com/movie/1"],
            "trailers": [],
            "category": null,
            "content_type": "movie",
            // a synonym, a name needing sanitization and a deleted tag
            "tags": ["scifi", "Sci-Fi", "Blu-Ray"],
            "tagline": null,
            "platform": null,
            "original_release_date": null,
            "original_release_date_only_year_known": true,
            "affiliated_artists": [],
            "series_id": null,
            "screenshots": [],
            "master_group_id": null
        },
        "edition_group": null,
        // the same artist twice, with a different role each time
        "affiliated_artists": [
            {"name": "Some Person", "aliases": [], "description": "", "pictures": [], "roles": ["director"], "nickname": null},
            {"name": "Some Person", "aliases": [], "description": "", "pictures": [], "roles": ["writer"], "nickname": null},
            {"name": "Another Person", "aliases": [], "description": "", "pictures": [], "roles": ["actor"], "nickname": "The Hero"}
        ]
    });
    let (plugin_url, _) = spawn_http_server("200 OK", scraped_data.to_string());

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app_with_plugin(
        pool,
        ExternalSourcePlugin {
            id: String::from("example"),
            placeholder: String::from("Example url"),
            sources: BTreeMap::from([(String::from("Example"), vec![ContentType::Movie])]),
            url: format!("{plugin_url}/scrape"),
            timeout_seconds: 30,
        },
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/external-sources/example?url=https://example.com/movie/1")
        .to_request();

    let external_db_data = call_and_read_body_json::<ExternalDBData, _>(&service, req).await;

    assert_eq!(external_db_data.title_group.tags, vec!["science.fiction"]);
    assert_eq!(
        external_db_data.title_group.external_links,
        vec!["https://example.com/movie/1"]
    );

    assert_eq!(external_db_data.affiliated_artists.len(), 2);
    assert_eq!(
        external_db_data.affiliated_artists[0].artist.name,
        "Some Person"
    );
    assert_eq!(
        external_db_data.affiliated_artists[0].roles,
        vec!["director", "writer"]
    );
    assert_eq!(
        external_db_data.affiliated_artists[1].nickname,
        Some(String::from("The Hero"))
    );
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_the_error_a_plugin_answers_with_is_given_to_the_uploader(pool: PgPool) {
    let (plugin_url, _) = spawn_http_server(
        "502 Bad Gateway",
        serde_json::json!({"error": "example.com is unreachable"}).to_string(),
    );

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app_with_plugin(
        pool,
        ExternalSourcePlugin {
            id: String::from("example"),
            placeholder: String::from("Example url"),
            sources: BTreeMap::from([(String::from("Example"), vec![ContentType::Movie])]),
            url: format!("{plugin_url}/scrape"),
            timeout_seconds: 30,
        },
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/external-sources/example?url=https://example.com/movie/1")
        .to_request();

    let error_response = call_and_read_body_json_with_status::<ErrorResponse, _>(
        &service,
        req,
        StatusCode::BAD_GATEWAY,
    )
    .await;

    assert_eq!(error_response.error, "example.com is unreachable");
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_upload_information_lists_the_websites_declared_by_a_plugin(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app_with_plugin(
        pool,
        ExternalSourcePlugin {
            id: String::from("example"),
            placeholder: String::from("Example url"),
            sources: BTreeMap::from([
                (String::from("Example"), vec![ContentType::Movie]),
                (
                    String::from("Other Example"),
                    vec![ContentType::Movie, ContentType::TVShow],
                ),
            ]),
            url: String::from("http://127.0.0.1:1/scrape"),
            timeout_seconds: 30,
        },
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrents/upload-info")
        .to_request();

    let upload_information = call_and_read_body_json::<UploadInformation, _>(&service, req).await;
    let plugin_source = upload_information
        .external_sources
        .iter()
        .find(|external_source| external_source.id == "example")
        .expect("the plugin source is missing from the upload information");

    assert_eq!(
        plugin_source.sources,
        BTreeMap::from([
            (String::from("Example"), vec![String::from("movie")]),
            (
                String::from("Other Example"),
                vec![String::from("movie"), String::from("tv_show")]
            ),
        ])
    );
    // a built in source accepts links from a single website, so the interface shows no tooltip
    let built_in_source = upload_information
        .external_sources
        .iter()
        .find(|external_source| external_source.id == "tmdb")
        .expect("the built in sources are missing from the upload information");
    assert_eq!(
        built_in_source.sources,
        BTreeMap::from([(
            String::from("TMDB"),
            vec![String::from("movie"), String::from("tv_show")]
        )])
    );
}
