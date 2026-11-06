pub mod common;
pub mod mocks;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{common::PaginatedResults, wiki::WikiSearchResult},
};
use common::{
    TestUser, auth_header, call_and_read_body_json_with_status, create_test_app_and_login,
};
use mocks::mock_redis::MockRedisPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_wiki_articles"),
    migrations = "../storage/migrations"
)]
async fn test_search_wiki_by_title(pool: sqlx::PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/wiki?search_string=guidelines&title_only=true&page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<WikiSearchResult> =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].title, "Upload Guidelines");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_wiki_articles"),
    migrations = "../storage/migrations"
)]
async fn test_search_wiki_title_only_does_not_match_body(pool: sqlx::PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // "torrents" only appears in the body of "Upload Guidelines", not in any title
    let req = test::TestRequest::get()
        .uri("/api/search/wiki?search_string=torrents&title_only=true&page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<WikiSearchResult> =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 0);
    assert!(response.results.is_empty());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_wiki_articles"),
    migrations = "../storage/migrations"
)]
async fn test_search_wiki_includes_body(pool: sqlx::PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // "torrents" appears in the body of "Upload Guidelines"
    let req = test::TestRequest::get()
        .uri("/api/search/wiki?search_string=torrents&title_only=false&page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<WikiSearchResult> =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].title, "Upload Guidelines");
}
