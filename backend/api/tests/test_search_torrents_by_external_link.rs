pub mod common;
pub mod mocks;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{common::PaginatedResults, title_group::TitleGroupHierarchyLite},
};
use common::{
    TestUser, auth_header, call_and_read_body_json_with_status, create_test_app_and_login,
};
use mocks::mock_redis::MockRedisPool;
use std::sync::Arc;

async fn search_by_external_link(
    service: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    token: &str,
    external_link: &str,
) -> PaginatedResults<TitleGroupHierarchyLite> {
    let request = test::TestRequest::get()
        .uri(&format!(
            "/api/search/torrents/lite?title_group_name={}&page=1&page_size=10&order_by_column=torrent_created_at&order_by_direction=desc&title_group_include_empty_groups=true",
            urlencoding::encode(external_link)
        ))
        .insert_header(auth_header(token))
        .to_request();

    call_and_read_body_json_with_status(service, request, StatusCode::OK).await
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_groups_with_similar_external_links",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_searching_by_external_link_only_matches_the_whole_link(pool: sqlx::PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // a link that is a prefix of another one must not match that other one
    let results = search_by_external_link(&service, &user.token, "https://example.com/1").await;
    assert_eq!(results.total_items, 1);
    assert_eq!(results.results[0].name, "Short Link Title Group");

    let results = search_by_external_link(&service, &user.token, "https://example.com/123").await;
    assert_eq!(results.total_items, 1);
    assert_eq!(results.results[0].name, "Long Link Title Group");

    // trailing slashes are ignored, on both the searched link and the stored one
    let results = search_by_external_link(&service, &user.token, "https://example.com/1/").await;
    assert_eq!(results.total_items, 1);
    assert_eq!(results.results[0].name, "Short Link Title Group");

    let results = search_by_external_link(&service, &user.token, "https://example.com/456").await;
    assert_eq!(results.total_items, 1);
    assert_eq!(results.results[0].name, "Trailing Slash Link Title Group");

    let results = search_by_external_link(&service, &user.token, "https://example.com/45").await;
    assert_eq!(results.total_items, 0);
}
