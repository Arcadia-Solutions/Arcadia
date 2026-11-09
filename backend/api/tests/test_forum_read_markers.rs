use crate::{common, mocks};

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::forum::{
    ForumPost, ForumPostHierarchy, ForumThread, ForumThreadEnriched, UserCreatedForumPost,
    UserCreatedForumThread,
};
use common::{TestUser, auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

// ============================================================================
// VIEWS COUNT TESTS
// ============================================================================

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category",
    "with_test_forum_thread",
    "with_test_forum_post"
))]
async fn test_views_count_increments_on_first_view(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Get thread before viewing posts - views should be 0
    let req = test::TestRequest::get()
        .uri("/api/forum/thread?id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let thread: ForumThreadEnriched =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(thread.views_count, 0);

    // View the thread's posts (this triggers the upsert)
    let req = test::TestRequest::get()
        .uri("/api/forum/thread/posts?thread_id=100&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();
    let _: PaginatedResults<ForumPostHierarchy> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Get thread again - views should be 1
    let req = test::TestRequest::get()
        .uri("/api/forum/thread?id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let thread: ForumThreadEnriched =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(thread.views_count, 1);

    // View posts again - views should still be 1 (same user)
    let req = test::TestRequest::get()
        .uri("/api/forum/thread/posts?thread_id=100&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();
    let _: PaginatedResults<ForumPostHierarchy> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let req = test::TestRequest::get()
        .uri("/api/forum/thread?id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let thread: ForumThreadEnriched =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(thread.views_count, 1);
}

// ============================================================================
// EVER_OPENED / HAS_NEW_POSTS TESTS
// ============================================================================

/// Helper to get `is_read` and `has_new_posts` for a thread from the sub-category endpoint.
/// The sub-category endpoint returns JSON built in SQL, so we parse as raw Value.
async fn get_thread_read_status<S>(
    service: &S,
    token: &str,
    sub_category_id: i32,
    thread_id: i64,
) -> (bool, bool)
where
    S: actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
{
    let req = test::TestRequest::get()
        .uri(&format!("/api/forum/sub-category?id={}", sub_category_id))
        .insert_header(auth_header(token))
        .to_request();

    let resp = test::call_service(service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let wrapper: serde_json::Value = serde_json::from_slice(&body).expect("valid JSON response");
    let data = &wrapper["data"];

    let threads = data["threads"]
        .as_array()
        .expect("threads should be an array");
    let thread = threads
        .iter()
        .find(|t| t["id"].as_i64() == Some(thread_id))
        .expect("thread should exist");

    let is_read = thread["ever_opened"].as_bool().unwrap_or(false);
    let has_new_posts = thread["has_new_posts"].as_bool().unwrap_or(true);
    (is_read, has_new_posts)
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category",
    "with_test_forum_thread",
    "with_test_forum_post"
))]
async fn test_thread_is_unread_before_viewing_posts(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let (is_read, has_new_posts) = get_thread_read_status(&service, &user.token, 100, 100).await;
    assert!(!is_read);
    assert!(has_new_posts);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category",
    "with_test_forum_thread",
    "with_test_forum_post"
))]
async fn test_thread_becomes_read_after_viewing_posts(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // View the thread's posts
    let req = test::TestRequest::get()
        .uri("/api/forum/thread/posts?thread_id=100&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();
    let _: PaginatedResults<ForumPostHierarchy> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Check that the thread is now marked as read with no new posts
    let (is_read, has_new_posts) = get_thread_read_status(&service, &user.token, 100, 100).await;
    assert!(is_read);
    assert!(!has_new_posts);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category"
))]
async fn test_thread_becomes_unread_after_new_post(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // Create a thread
    let create_body = UserCreatedForumThread {
        forum_sub_category_id: 100,
        name: "Read Marker Test Thread".into(),
        first_post: UserCreatedForumPost {
            content: "First post".into(),
            forum_thread_id: 0,
        },
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/thread")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();
    let thread: ForumThread =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // View posts to mark as read
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/forum/thread/posts?thread_id={}&page_size=10",
            thread.id
        ))
        .insert_header(auth_header(&user.token))
        .to_request();
    let _: PaginatedResults<ForumPostHierarchy> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Thread should be read now with no new posts
    let (is_read, has_new_posts) =
        get_thread_read_status(&service, &user.token, 100, thread.id).await;
    assert!(is_read);
    assert!(!has_new_posts);

    // Another user adds a new post
    let (service2, other_user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditArtist).await;

    let post_body = UserCreatedForumPost {
        content: "New post from another user".into(),
        forum_thread_id: thread.id,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/post")
        .insert_header(auth_header(&other_user.token))
        .set_json(&post_body)
        .to_request();
    let _: ForumPost =
        common::call_and_read_body_json_with_status(&service2, req, StatusCode::CREATED).await;

    // Original user checks again - thread was read but has new posts
    let (ever_opened, has_new_posts) =
        get_thread_read_status(&service, &user.token, 100, thread.id).await;
    assert!(ever_opened);
    assert!(has_new_posts);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category"
))]
async fn test_has_new_posts_clears_after_rereading(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // Create a thread and view its posts
    let create_body = UserCreatedForumThread {
        forum_sub_category_id: 100,
        name: "Reread Test Thread".into(),
        first_post: UserCreatedForumPost {
            content: "First post".into(),
            forum_thread_id: 0,
        },
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/thread")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();
    let thread: ForumThread =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/forum/thread/posts?thread_id={}&page_size=10",
            thread.id
        ))
        .insert_header(auth_header(&user.token))
        .to_request();
    let _: PaginatedResults<ForumPostHierarchy> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Another user adds a new post
    let (service2, other_user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditArtist).await;

    let post_body = UserCreatedForumPost {
        content: "New post".into(),
        forum_thread_id: thread.id,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/post")
        .insert_header(auth_header(&other_user.token))
        .set_json(&post_body)
        .to_request();
    let _: ForumPost =
        common::call_and_read_body_json_with_status(&service2, req, StatusCode::CREATED).await;

    // has_new_posts should be true
    let (ever_opened, has_new_posts) =
        get_thread_read_status(&service, &user.token, 100, thread.id).await;
    assert!(ever_opened);
    assert!(has_new_posts);

    // Original user re-reads the thread posts
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/forum/thread/posts?thread_id={}&page_size=10",
            thread.id
        ))
        .insert_header(auth_header(&user.token))
        .to_request();
    let _: PaginatedResults<ForumPostHierarchy> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // has_new_posts should be cleared
    let (ever_opened, has_new_posts) =
        get_thread_read_status(&service, &user.token, 100, thread.id).await;
    assert!(ever_opened);
    assert!(!has_new_posts);
}
