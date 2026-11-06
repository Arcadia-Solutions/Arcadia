pub mod common;
pub mod mocks;

use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::conversation::ConversationSearchResult;
use common::{TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_searchable_conversations"),
    migrations = "../storage/migrations"
)]
async fn test_search_conversations_no_filter(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/conversations?page=1&page_size=50&search_titles_only=true&order_by_column=last_message&order_by_direction=desc&all_conversations=false")
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<ConversationSearchResult> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.results.len(), 3);
    assert_eq!(results.total_items, 3);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_searchable_conversations"),
    migrations = "../storage/migrations"
)]
async fn test_search_conversations_by_subject(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // "jazz" appears in subjects of conversations 200 and 202
    let req = test::TestRequest::get()
        .uri("/api/search/conversations?search_term=jazz&search_titles_only=true&page=1&page_size=50&order_by_column=last_message&order_by_direction=desc&all_conversations=false")
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<ConversationSearchResult> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].subject, "Jazz collection");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_searchable_conversations"),
    migrations = "../storage/migrations"
)]
async fn test_search_conversations_by_content(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // "jazz" in titles: conversation 202 ("Jazz collection")
    // "jazz" in message content: conversation 200 ("Have you listened to any good jazz albums lately?")
    // With search_titles_only=false, both should match
    let req = test::TestRequest::get()
        .uri("/api/search/conversations?search_term=jazz&search_titles_only=false&page=1&page_size=50&order_by_column=last_message&order_by_direction=desc&all_conversations=false")
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<ConversationSearchResult> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.results.len(), 2);

    let subjects: Vec<&str> = results.results.iter().map(|r| r.subject.as_str()).collect();
    assert!(subjects.contains(&"Jazz collection"));
    assert!(subjects.contains(&"Music recommendations"));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_searchable_conversations"),
    migrations = "../storage/migrations"
)]
async fn test_search_conversations_content_only_match(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // "seeding" only in message content of conversation 201, not in any subject
    // titles_only=true should return nothing
    let req = test::TestRequest::get()
        .uri("/api/search/conversations?search_term=seeding&search_titles_only=true&page=1&page_size=50&order_by_column=last_message&order_by_direction=desc&all_conversations=false")
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<ConversationSearchResult> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.results.len(), 0);
    assert_eq!(results.total_items, 0);

    // titles_only=false should find it
    let req = test::TestRequest::get()
        .uri("/api/search/conversations?search_term=seeding&search_titles_only=false&page=1&page_size=50&order_by_column=last_message&order_by_direction=desc&all_conversations=false")
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<ConversationSearchResult> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.total_items, 1);
    assert_eq!(results.results[0].subject, "Upload help");
}
