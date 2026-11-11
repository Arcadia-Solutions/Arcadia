use crate::{common, mocks};

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use common::{auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users", "with_deletable_title_group"))]
async fn test_user_with_permission_can_delete_title_group_without_torrents(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteTitleGroup,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/title-groups?title_group_id=3")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify title group was deleted
    let result = pool.find_title_group(3).await;
    assert!(result.is_err());
}

#[sqlx::test(fixtures("with_test_users", "with_deletable_title_group"))]
async fn test_user_with_permission_can_delete_title_group_with_only_deleted_torrents(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteTitleGroup,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/title-groups?title_group_id=4")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify title group was deleted
    let result = pool.find_title_group(4).await;
    assert!(result.is_err());
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent"
))]
async fn test_cannot_delete_title_group_with_undeleted_torrents(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteTitleGroup,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/title-groups?title_group_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Verify title group was NOT deleted
    let result = pool.find_title_group(1).await;
    assert!(result.is_ok());
}

#[sqlx::test(fixtures("with_test_users", "with_deletable_title_group"))]
async fn test_user_without_permission_cannot_delete_title_group(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .uri("/api/title-groups?title_group_id=3")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Verify title group was NOT deleted
    let result = pool.find_title_group(3).await;
    assert!(result.is_ok());
}

#[sqlx::test(fixtures("with_test_users", "with_deletable_title_group"))]
async fn test_deleting_title_group_decrements_artist_counters(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteTitleGroup,
    )
    .await;

    // Artist 2 starts with title_groups_amount=1, edition_groups_amount=2, torrents_amount=0
    let artist_before = pool.find_artist_by_id(2).await.unwrap();
    assert_eq!(artist_before.title_groups_amount, 1);
    assert_eq!(artist_before.edition_groups_amount, 2);
    assert_eq!(artist_before.torrents_amount, 0);

    let req = test::TestRequest::delete()
        .uri("/api/title-groups?title_group_id=3")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // All artist counters should be decremented
    let artist_after = pool.find_artist_by_id(2).await.unwrap();
    assert_eq!(artist_after.title_groups_amount, 0);
    assert_eq!(artist_after.edition_groups_amount, 0);
    assert_eq!(artist_after.torrents_amount, 0);
}
