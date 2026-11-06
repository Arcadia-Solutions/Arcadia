pub mod common;
pub mod mocks;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{common::PaginatedResults, unauthorized_access::UnauthorizedAccess},
};
use chrono::Utc;
use common::{TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;
use urlencoding::encode;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_without_permission_cannot_search_unauthorized_access(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let from_date_str = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let to_date_str = Utc::now().to_rfc3339();
    let from_date = encode(&from_date_str);
    let to_date = encode(&to_date_str);

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri(&format!(
            "/api/unauthorized-access?from_date={}&to_date={}&sort_by_column=created_at&sort_by_direction=desc&page=1&page_size=50",
            from_date, to_date
        ))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_with_permission_can_search_unauthorized_access(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUnauthorizedAccess,
    )
    .await;

    let from_date_str = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let to_date_str = Utc::now().to_rfc3339();
    let from_date = encode(&from_date_str);
    let to_date = encode(&to_date_str);

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri(&format!(
            "/api/unauthorized-access?from_date={}&to_date={}&sort_by_column=created_at&sort_by_direction=desc&page=1&page_size=50",
            from_date, to_date
        ))
        .to_request();

    let result =
        call_and_read_body_json::<PaginatedResults<UnauthorizedAccess>, _>(&service, req).await;

    assert_eq!(result.page, 1);
    assert_eq!(result.page_size, 50);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_search_filters_by_user_id(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUnauthorizedAccess,
    )
    .await;

    let from_date_str = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let to_date_str = Utc::now().to_rfc3339();
    let from_date = encode(&from_date_str);
    let to_date = encode(&to_date_str);

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri(&format!(
            "/api/unauthorized-access?user_id=100&from_date={}&to_date={}&sort_by_column=created_at&sort_by_direction=desc&page=1&page_size=50",
            from_date, to_date
        ))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_search_filters_by_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUnauthorizedAccess,
    )
    .await;

    let from_date_str = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let to_date_str = Utc::now().to_rfc3339();
    let from_date = encode(&from_date_str);
    let to_date = encode(&to_date_str);

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri(&format!(
            "/api/unauthorized-access?permission=edit_artist&from_date={}&to_date={}&sort_by_column=created_at&sort_by_direction=desc&page=1&page_size=50",
            from_date, to_date
        ))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_search_sorts_by_permission_column(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUnauthorizedAccess,
    )
    .await;

    let from_date_str = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let to_date_str = Utc::now().to_rfc3339();
    let from_date = encode(&from_date_str);
    let to_date = encode(&to_date_str);

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri(&format!(
            "/api/unauthorized-access?from_date={}&to_date={}&sort_by_column=missing_permission&sort_by_direction=asc&page=1&page_size=50",
            from_date, to_date
        ))
        .to_request();

    let result =
        call_and_read_body_json::<PaginatedResults<UnauthorizedAccess>, _>(&service, req).await;

    assert_eq!(result.page, 1);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_search_pagination_works(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUnauthorizedAccess,
    )
    .await;

    let from_date_str = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let to_date_str = Utc::now().to_rfc3339();
    let from_date = encode(&from_date_str);
    let to_date = encode(&to_date_str);

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri(&format!(
            "/api/unauthorized-access?from_date={}&to_date={}&sort_by_column=created_at&sort_by_direction=desc&page=2&page_size=10",
            from_date, to_date
        ))
        .to_request();

    let result =
        call_and_read_body_json::<PaginatedResults<UnauthorizedAccess>, _>(&service, req).await;

    assert_eq!(result.page, 2);
    assert_eq!(result.page_size, 10);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_search_respects_page_size_limit(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUnauthorizedAccess,
    )
    .await;

    let from_date_str = (Utc::now() - chrono::Duration::days(30)).to_rfc3339();
    let to_date_str = Utc::now().to_rfc3339();
    let from_date = encode(&from_date_str);
    let to_date = encode(&to_date_str);

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri(&format!(
            "/api/unauthorized-access?from_date={}&to_date={}&sort_by_column=created_at&sort_by_direction=desc&page=1&page_size=200",
            from_date, to_date
        ))
        .to_request();

    let result =
        call_and_read_body_json::<PaginatedResults<UnauthorizedAccess>, _>(&service, req).await;

    assert_eq!(result.page_size, 100);
}
