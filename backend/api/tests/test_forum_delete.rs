pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use common::{auth_header, create_test_app_and_login, TestUser};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

// ============================================================================
// DELETE SUB-CATEGORY TESTS
// ============================================================================

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_delete_empty_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteForumSubCategory,
    )
    .await;

    // Sub-category 100 exists and has no threads
    let req = test::TestRequest::delete()
        .uri("/api/forum/sub-category?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify deletion by trying to delete again (should fail)
    let req2 = test::TestRequest::delete()
        .uri("/api/forum/sub-category?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp2 = test::call_service(&service, req2).await;
    assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post"
    ),
    migrations = "../storage/migrations"
)]
async fn test_delete_thread_decrements_author_forum_posts(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    pool.recompute_cached_amounts().await.unwrap();
    let (service, staff) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteForumThread,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/forum/thread?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // user 100 authored the 4 fixture posts, one of them in thread 100
    let author = pool.find_user_profile(&100).await.unwrap();
    assert_eq!(author.forum_posts, Some(3));
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread"
    ),
    migrations = "../storage/migrations"
)]
async fn test_cannot_delete_sub_category_with_threads(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::DeleteForumSubCategory,
    )
    .await;

    // Sub-category 100 has threads
    let req = test::TestRequest::delete()
        .uri("/api/forum/sub-category?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_non_staff_cannot_delete_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .uri("/api/forum/sub-category?id=100")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ============================================================================
// DELETE THREAD TESTS (CASCADE DELETE & COUNTER UPDATES)
// ============================================================================

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_delete_thread_with_posts(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteForumThread,
    )
    .await;

    // Delete thread 100 (which has posts - cascade delete should handle them)
    let req = test::TestRequest::delete()
        .uri("/api/forum/thread?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify deletion by trying to delete again (should fail)
    let req2 = test::TestRequest::delete()
        .uri("/api/forum/thread?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp2 = test::call_service(&service, req2).await;
    assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread"
    ),
    migrations = "../storage/migrations"
)]
async fn test_non_staff_cannot_delete_thread(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .uri("/api/forum/thread?id=100")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread"
    ),
    migrations = "../storage/migrations"
)]
async fn test_delete_thread_without_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = common::create_test_app(pool, MockRedisPool::default()).await;

    let req = test::TestRequest::delete()
        .uri("/api/forum/thread?id=100")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// DELETE POST TESTS
// ============================================================================

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_delete_post(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteForumPost,
    )
    .await;

    // Delete post 100
    let req = test::TestRequest::delete()
        .uri("/api/forum/post?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify deletion by trying to delete again (should fail)
    let req2 = test::TestRequest::delete()
        .uri("/api/forum/post?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp2 = test::call_service(&service, req2).await;
    assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post"
    ),
    migrations = "../storage/migrations"
)]
async fn test_non_staff_cannot_delete_post(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .uri("/api/forum/post?id=100")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post"
    ),
    migrations = "../storage/migrations"
)]
async fn test_delete_post_without_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = common::create_test_app(pool, MockRedisPool::default()).await;

    let req = test::TestRequest::delete()
        .uri("/api/forum/post?id=100")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_delete_nonexistent_post(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::DeleteForumPost).await;

    let req = test::TestRequest::delete()
        .uri("/api/forum/post?id=999")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
