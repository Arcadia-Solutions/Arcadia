use crate::{common, mocks};

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::forum::{
    EditedForumSubCategory, ForumSubCategory, ForumThreadSortBy, ForumThreadSortDirection,
    UserCreatedForumSubCategory,
};
use common::{TestUser, auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

// ============================================================================
// CREATE SUB-CATEGORY TESTS
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_category"),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_create_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::CreateForumSubCategory,
    )
    .await;

    let create_body = UserCreatedForumSubCategory {
        forum_category_id: 100,
        name: "New Sub Category".into(),
        new_threads_restricted: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&create_body)
        .to_request();

    let sub_category: ForumSubCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    assert_eq!(sub_category.name, "New Sub Category");
    assert_eq!(sub_category.forum_category_id, 100);
    assert_eq!(sub_category.created_by_id, 109);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_category"),
    migrations = "../storage/migrations"
)]
async fn test_non_staff_cannot_create_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let create_body = UserCreatedForumSubCategory {
        forum_category_id: 100,
        name: "New Sub Category".into(),
        new_threads_restricted: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_category"),
    migrations = "../storage/migrations"
)]
async fn test_create_sub_category_without_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = common::create_test_app(pool, MockRedisPool::default()).await;

    let create_body = UserCreatedForumSubCategory {
        forum_category_id: 100,
        name: "New Sub Category".into(),
        new_threads_restricted: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/sub-category")
        .set_json(&create_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_category"),
    migrations = "../storage/migrations"
)]
async fn test_create_sub_category_with_empty_name(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::CreateForumSubCategory,
    )
    .await;

    let create_body = UserCreatedForumSubCategory {
        forum_category_id: 100,
        name: "".into(),
        new_threads_restricted: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&create_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_category"),
    migrations = "../storage/migrations"
)]
async fn test_create_sub_category_with_whitespace_only_name(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::CreateForumSubCategory,
    )
    .await;

    let create_body = UserCreatedForumSubCategory {
        forum_category_id: 100,
        name: "   ".into(),
        new_threads_restricted: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&create_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// EDIT SUB-CATEGORY TESTS
// ============================================================================

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_edit_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditForumSubCategory,
    )
    .await;

    let edit_body = EditedForumSubCategory {
        id: 100,
        name: "Updated Sub Category Name".into(),
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

    assert_eq!(sub_category.id, 100);
    assert_eq!(sub_category.name, "Updated Sub Category Name");
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_non_staff_cannot_edit_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let edit_body = EditedForumSubCategory {
        id: 100,
        name: "Updated Sub Category Name".into(),
        new_threads_restricted: false,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&user.token))
        .set_json(&edit_body)
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
async fn test_edit_sub_category_without_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = common::create_test_app(pool, MockRedisPool::default()).await;

    let edit_body = EditedForumSubCategory {
        id: 100,
        name: "Updated Sub Category Name".into(),
        new_threads_restricted: false,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .set_json(&edit_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_category"),
    migrations = "../storage/migrations"
)]
async fn test_edit_nonexistent_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditForumSubCategory,
    )
    .await;

    let edit_body = EditedForumSubCategory {
        id: 999,
        name: "Updated Sub Category Name".into(),
        new_threads_restricted: false,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_edit_sub_category_with_empty_name(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditForumSubCategory,
    )
    .await;

    let edit_body = EditedForumSubCategory {
        id: 100,
        name: "".into(),
        new_threads_restricted: false,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
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
async fn test_edit_sub_category_with_whitespace_only_name(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditForumSubCategory,
    )
    .await;

    let edit_body = EditedForumSubCategory {
        id: 100,
        name: "   ".into(),
        new_threads_restricted: false,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// THREAD SORT ORDER TESTS
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_category"),
    migrations = "../storage/migrations"
)]
async fn test_new_sub_category_defaults_to_latest_post_sort(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::CreateForumSubCategory,
    )
    .await;

    let create_body = UserCreatedForumSubCategory {
        forum_category_id: 100,
        name: "New Sub Category".into(),
        new_threads_restricted: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&create_body)
        .to_request();

    let sub_category: ForumSubCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    assert!(matches!(
        sub_category.thread_sort_by,
        ForumThreadSortBy::LatestPost
    ));
    assert!(matches!(
        sub_category.thread_sort_direction,
        ForumThreadSortDirection::Descending
    ));
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_change_thread_sort_direction(pool: PgPool) {
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
        new_threads_restricted: false,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Ascending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
        .to_request();

    let sub_category: ForumSubCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert!(matches!(
        sub_category.thread_sort_direction,
        ForumThreadSortDirection::Ascending
    ));
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_change_thread_sort_by(pool: PgPool) {
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
        new_threads_restricted: false,
        thread_sort_by: ForumThreadSortBy::PostsAmount,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
        .to_request();

    let sub_category: ForumSubCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert!(matches!(
        sub_category.thread_sort_by,
        ForumThreadSortBy::PostsAmount
    ));
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_category"),
    migrations = "../storage/migrations"
)]
async fn test_create_and_edit_sub_category_flow(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ForumSubCategoryFlow,
    )
    .await;

    // Create a sub category
    let create_body = UserCreatedForumSubCategory {
        forum_category_id: 100,
        name: "Initial Sub Category".into(),
        new_threads_restricted: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&create_body)
        .to_request();

    let sub_category: ForumSubCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    let sub_category_id = sub_category.id;

    assert_eq!(sub_category.name, "Initial Sub Category");
    assert_eq!(sub_category.forum_category_id, 100);

    // Edit the sub category
    let edit_body = EditedForumSubCategory {
        id: sub_category_id,
        name: "Edited Sub Category".into(),
        new_threads_restricted: false,
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
    };

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
        .to_request();

    let edited_sub_category: ForumSubCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(edited_sub_category.id, sub_category_id);
    assert_eq!(edited_sub_category.name, "Edited Sub Category");
}
