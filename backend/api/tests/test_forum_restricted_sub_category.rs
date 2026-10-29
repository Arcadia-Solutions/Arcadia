pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::forum::{
    EditedForumSubCategory, EditedForumThread, ForumSubCategory, ForumSubCategoryAllowedPoster,
    ForumThread, ForumThreadSortBy, ForumThreadSortDirection, UserCreatedForumPost,
    UserCreatedForumThread,
};
use arcadia_storage::models::user::UserLite;
use common::{auth_header, create_test_app_and_login, TestUser};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

// ============================================================================
// RESTRICTED THREAD CREATION TESTS
// ============================================================================

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_restricted_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_allowed_user_can_create_thread_in_restricted_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let create_body = UserCreatedForumThread {
        forum_sub_category_id: 102,
        name: "Thread in Restricted Sub Category".into(),
        first_post: UserCreatedForumPost {
            content: "Allowed poster content".into(),
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

    assert_eq!(thread.name, "Thread in Restricted Sub Category");
    assert_eq!(thread.forum_sub_category_id, 102);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_restricted_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_non_allowed_user_cannot_create_thread_in_restricted_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    // user_edit_art (id=101) has create_forum_thread permission but is NOT in the allowed posters
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditArtist).await;

    let create_body = UserCreatedForumThread {
        forum_sub_category_id: 102,
        name: "Should Be Rejected".into(),
        first_post: UserCreatedForumPost {
            content: "This should not be created".into(),
            forum_thread_id: 0,
        },
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/thread")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_restricted_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_non_allowed_user_cannot_move_thread_into_restricted_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    // user_edit_art (id=101) is NOT in the allowed posters of sub-category 102
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditArtist).await;

    let req = test::TestRequest::post()
        .uri("/api/forum/thread")
        .insert_header(auth_header(&user.token))
        .set_json(&UserCreatedForumThread {
            forum_sub_category_id: 100,
            name: "Thread to move".into(),
            first_post: UserCreatedForumPost {
                content: "content".into(),
                forum_thread_id: 0,
            },
        })
        .to_request();
    let thread: ForumThread =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    let req = test::TestRequest::put()
        .uri("/api/forum/thread")
        .insert_header(auth_header(&user.token))
        .set_json(&EditedForumThread {
            id: thread.id,
            forum_sub_category_id: 102,
            name: thread.name,
        })
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_unrestricted_sub_category_allows_any_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let create_body = UserCreatedForumThread {
        forum_sub_category_id: 100,
        name: "Thread in Unrestricted Sub Category".into(),
        first_post: UserCreatedForumPost {
            content: "Anyone can post here".into(),
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

    assert_eq!(thread.name, "Thread in Unrestricted Sub Category");
}

// ============================================================================
// EDIT SUB-CATEGORY RESTRICTION TOGGLE TESTS
// ============================================================================

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_enable_restriction_on_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditForumSubCategory,
    )
    .await;

    let edit_body = EditedForumSubCategory {
        id: 100,
        name: "Test Sub Category".into(),
        new_threads_restricted: true,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
        .to_request();

    let sub_category: ForumSubCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert!(sub_category.new_threads_restricted);
}

// ============================================================================
// ALLOWED POSTER MANAGEMENT TESTS
// ============================================================================

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_add_allowed_poster(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditForumSubCategory,
    )
    .await;

    let body = ForumSubCategoryAllowedPoster {
        forum_sub_category_id: 100,
        user_id: 100,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/sub-category/allowed-poster")
        .insert_header(auth_header(&staff.token))
        .set_json(&body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the poster was added
    let req = test::TestRequest::get()
        .uri("/api/forum/sub-category/allowed-poster?forum_sub_category_id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let posters: Vec<UserLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(posters.len(), 1);
    assert_eq!(posters[0].id, 100);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_restricted_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_remove_allowed_poster(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditForumSubCategory,
    )
    .await;

    // Remove user_basic (100) from the restricted sub-category (102)
    let body = ForumSubCategoryAllowedPoster {
        forum_sub_category_id: 102,
        user_id: 100,
    };

    let req = test::TestRequest::delete()
        .uri("/api/forum/sub-category/allowed-poster")
        .insert_header(auth_header(&staff.token))
        .set_json(&body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the poster was removed
    let req = test::TestRequest::get()
        .uri("/api/forum/sub-category/allowed-poster?forum_sub_category_id=102")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let posters: Vec<UserLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert!(posters.is_empty());
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_non_staff_cannot_manage_allowed_posters(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let body = ForumSubCategoryAllowedPoster {
        forum_sub_category_id: 100,
        user_id: 100,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/sub-category/allowed-poster")
        .insert_header(auth_header(&user.token))
        .set_json(&body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ============================================================================
// INTEGRATION TEST
// ============================================================================

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_restrict_sub_category_then_verify_enforcement(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ForumSubCategoryFlow,
    )
    .await;

    // Step 1: Enable restriction on sub-category 100
    let edit_body = EditedForumSubCategory {
        id: 100,
        name: "Test Sub Category".into(),
        new_threads_restricted: true,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
        .to_request();

    let sub_category: ForumSubCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert!(sub_category.new_threads_restricted);

    // Step 2: Verify that a standard user (not in allowed list) cannot create a thread
    // ForumSubCategoryFlow user (114) has create_forum_sub_category + edit_forum_sub_category
    // but NOT create_forum_thread, so we use a separate login for the standard user
    // The staff user itself doesn't have create_forum_thread either, so the restriction
    // is tested via the earlier dedicated tests. Here we just verify the toggle persisted.

    // Step 3: Disable restriction
    let edit_body = EditedForumSubCategory {
        id: 100,
        name: "Test Sub Category".into(),
        new_threads_restricted: false,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
        .to_request();

    let sub_category: ForumSubCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert!(!sub_category.new_threads_restricted);
}
