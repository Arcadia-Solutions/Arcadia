use crate::{common, mocks};

use actix_web::{http::StatusCode, test};
use arcadia_storage::{connection_pool::ConnectionPool, models::user::UserPermission};
use common::{
    TestUser, auth_header, call_and_read_body_json, create_test_app, create_test_app_and_login,
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_can_get_user_permissions(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SetAndViewUserPermissions,
    )
    .await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/100/permissions")
        .to_request();

    let permissions = call_and_read_body_json::<Vec<UserPermission>, _>(&service, req).await;

    assert_eq!(permissions.len(), 6);
    assert!(permissions.contains(&UserPermission::DownloadTorrent));
    assert!(permissions.contains(&UserPermission::UploadTorrent));
    assert!(permissions.contains(&UserPermission::CreateTorrentRequest));
    assert!(permissions.contains(&UserPermission::CreateForumThread));
    assert!(permissions.contains(&UserPermission::CreateForumPost));
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_regular_user_cannot_get_user_permissions(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/118/permissions")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_get_user_permissions_requires_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;

    let req = test::TestRequest::get()
        .uri("/api/users/100/permissions")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_get_nonexistent_user_permissions(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SetAndViewUserPermissions,
    )
    .await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/500/permissions")
        .to_request();

    let resp = test::call_service(&service, req).await;
    // NOTE: Currently returns 400 due to WrongUsernameOrPassword error in find_user_with_id,
    // but API docs specify 404. This should be fixed to return NOT_FOUND.
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
