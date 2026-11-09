use crate::{common, mocks};

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::common::{OrderByDirection, PaginatedResults};
use arcadia_storage::models::user::{
    SearchUsersQuery, UserLite, UserPermission, UserPermissionMatchMode, UserSearchOrderBy,
    UserSearchResult,
};
use common::auth_header;
use common::create_test_app_and_login;
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

fn search_users_query() -> SearchUsersQuery {
    SearchUsersQuery {
        username: None,
        registered_after: None,
        registered_before: None,
        permissions: None,
        permissions_match: None,
        order_by: UserSearchOrderBy::Username,
        order_by_direction: OrderByDirection::Asc,
        page: 1,
        page_size: 20,
    }
}

fn search_users_request(token: &str, query: SearchUsersQuery) -> actix_http::Request {
    test::TestRequest::post()
        .uri("/api/search/users")
        .insert_header(auth_header(token))
        .set_json(query)
        .to_request()
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_lite_returns_results(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/users/lite?username=alice")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: Vec<UserLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Should find both alice_smith and alice_wonder
    assert_eq!(response.len(), 2);
    assert!(response.iter().any(|u| u.username == "alice_smith"));
    assert!(response.iter().any(|u| u.username == "alice_wonder"));
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_lite_case_insensitive(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/users/lite?username=ALICE")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: Vec<UserLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Should find both alice_smith and alice_wonder regardless of case
    assert_eq!(response.len(), 2);
    assert!(response.iter().any(|u| u.username == "alice_smith"));
    assert!(response.iter().any(|u| u.username == "alice_wonder"));
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_lite_partial_match(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/users/lite?username=brown")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: Vec<UserLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Should find charlie_brown
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].username, "charlie_brown");
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_lite_returns_user_lite_fields(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/users/lite?username=frank")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: Vec<UserLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.len(), 1);
    let frank = &response[0];
    assert_eq!(frank.username, "frank_miller");
    assert!(frank.warned);
    assert!(!frank.banned);
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_lite_returns_banned_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/users/lite?username=grace")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: Vec<UserLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.len(), 1);
    let grace = &response[0];
    assert_eq!(grace.username, "grace_hopper");
    assert!(!grace.warned);
    assert!(grace.banned);
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_lite_no_results(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/users/lite?username=nonexistent")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: Vec<UserLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.len(), 0);
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_without_permission_is_forbidden(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = search_users_request(
        &user.token,
        SearchUsersQuery {
            username: Some("alice".to_string()),
            ..search_users_query()
        },
    );

    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_with_permission_returns_results(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SearchUsers).await;

    let req = search_users_request(
        &user.token,
        SearchUsersQuery {
            username: Some("alice".to_string()),
            ..search_users_query()
        },
    );

    let response: PaginatedResults<UserSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 2);
    assert_eq!(response.results.len(), 2);
    assert!(response.results.iter().any(|u| u.username == "alice_smith"));
    assert!(
        response
            .results
            .iter()
            .any(|u| u.username == "alice_wonder")
    );
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_registration_date_range_filter(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SearchUsers).await;

    // The test users are registered "now", so a range ending in the far future keeps them...
    let req = search_users_request(
        &user.token,
        SearchUsersQuery {
            username: Some("alice".to_string()),
            registered_before: Some("2999-01-01T00:00:00Z".parse().unwrap()),
            ..search_users_query()
        },
    );
    let response: PaginatedResults<UserSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(response.total_items, 2);

    // ...while requiring registration after the far future excludes everyone.
    let req = search_users_request(
        &user.token,
        SearchUsersQuery {
            username: Some("alice".to_string()),
            registered_after: Some("2999-01-01T00:00:00Z".parse().unwrap()),
            ..search_users_query()
        },
    );
    let response: PaginatedResults<UserSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(response.total_items, 0);
    assert_eq!(response.results.len(), 0);
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_with_permissions"))]
async fn test_search_users_filters_by_any_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUsersWithPermissions,
    )
    .await;

    let req = search_users_request(
        &user.token,
        SearchUsersQuery {
            username: Some("perm_".to_string()),
            permissions: Some(vec![
                UserPermission::UploadTorrent,
                UserPermission::DownloadTorrent,
            ]),
            permissions_match: Some(UserPermissionMatchMode::Any),
            ..search_users_query()
        },
    );
    let response: PaginatedResults<UserSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 3);
    let mut usernames: Vec<&str> = response
        .results
        .iter()
        .map(|user| user.username.as_str())
        .collect();
    usernames.sort_unstable();
    assert_eq!(usernames, ["perm_both", "perm_download", "perm_upload"]);
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_with_permissions"))]
async fn test_search_users_filters_by_all_permissions(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUsersWithPermissions,
    )
    .await;

    let req = search_users_request(
        &user.token,
        SearchUsersQuery {
            username: Some("perm_".to_string()),
            permissions: Some(vec![
                UserPermission::UploadTorrent,
                UserPermission::DownloadTorrent,
            ]),
            permissions_match: Some(UserPermissionMatchMode::All),
            ..search_users_query()
        },
    );
    let response: PaginatedResults<UserSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].username, "perm_both");
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_with_permissions"))]
async fn test_search_users_permission_filter_requires_set_and_view_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SearchUsers).await;

    let req = search_users_request(
        &user.token,
        SearchUsersQuery {
            permissions: Some(vec![UserPermission::UploadTorrent]),
            ..search_users_query()
        },
    );

    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users", "with_test_users_for_search"))]
async fn test_search_users_lite_limits_to_five_results(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/users/lite?username=_")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: Vec<UserLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Should return at most 5 results even though there are more users with "_" in their username
    assert!(response.len() <= 5);
}
