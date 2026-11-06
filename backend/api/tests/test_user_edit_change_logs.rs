pub mod common;
pub mod mocks;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{common::PaginatedResults, user_edit_change_log::UserEditChangeLogResult},
};
use common::{TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_without_permission_cannot_search_user_edit_change_logs(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs?sort_by_column=edited_at&sort_by_direction=desc&page=1&page_size=50")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_with_permission_can_search_user_edit_change_logs(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUserEditChangeLogs,
    )
    .await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs?sort_by_column=edited_at&sort_by_direction=desc&page=1&page_size=50")
        .to_request();

    let result =
        call_and_read_body_json::<PaginatedResults<UserEditChangeLogResult>, _>(&service, req)
            .await;

    assert_eq!(result.page, 1);
    assert_eq!(result.page_size, 50);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_search_filters_by_user_id(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUserEditChangeLogs,
    )
    .await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs?user_id=100&sort_by_column=edited_at&sort_by_direction=desc&page=1&page_size=50")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_search_filters_by_item_type(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SearchUserEditChangeLogs,
    )
    .await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs?item_type=artist&sort_by_column=edited_at&sort_by_direction=desc&page=1&page_size=50")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// --- Delete single user edit change log ---

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_without_permission_cannot_delete_user_edit_change_log(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs?id=1")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_edit_change_logs"),
    migrations = "../storage/migrations"
)]
async fn test_user_with_permission_can_delete_user_edit_change_log(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteUserEditChangeLog,
    )
    .await;

    // Delete a specific change log entry
    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs?id=1")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the entry was actually deleted by searching
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs?sort_by_column=edited_at&sort_by_direction=desc&page=1&page_size=50")
        .to_request();

    let result =
        call_and_read_body_json::<PaginatedResults<UserEditChangeLogResult>, _>(&service, req)
            .await;

    assert_eq!(result.total_items, 2);
    assert!(result.results.iter().all(|log| log.id != 1));
}

// --- Delete all user edit change logs ---

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_without_permission_cannot_delete_all_user_edit_change_logs(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs/all")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_edit_change_logs"),
    migrations = "../storage/migrations"
)]
async fn test_user_with_permission_can_delete_all_user_edit_change_logs(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteUserEditChangeLog,
    )
    .await;

    // Delete all change log entries
    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs/all")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify all entries were deleted by searching
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-edit-change-logs?sort_by_column=edited_at&sort_by_direction=desc&page=1&page_size=50")
        .to_request();

    let result =
        call_and_read_body_json::<PaginatedResults<UserEditChangeLogResult>, _>(&service, req)
            .await;

    assert_eq!(result.total_items, 0);
    assert!(result.results.is_empty());
}
