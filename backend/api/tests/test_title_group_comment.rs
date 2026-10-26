pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::{
    common::PaginatedResults,
    title_group_comment::{
        EditedTitleGroupComment, TitleGroupComment, TitleGroupCommentSearchResult,
        UserCreatedTitleGroupComment,
    },
};
use common::{auth_header, create_test_app_and_login, TestUser};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_owner_can_edit_their_comment(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Create comment
    let create_body = UserCreatedTitleGroupComment {
        content: "Original comment".into(),
        title_group_id: 1,
        refers_to_torrent_id: None,
        answers_to_comment_id: None,
    };

    let req = test::TestRequest::post()
        .uri("/api/title-groups/comments")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();

    let comment: TitleGroupComment =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // Edit comment
    let edit_body = EditedTitleGroupComment {
        content: "Edited comment".into(),
        locked: false,
    };

    let req = test::TestRequest::put()
        .uri(&format!("/api/title-groups/comments/{}", comment.id))
        .insert_header(auth_header(&user.token))
        .set_json(&edit_body)
        .to_request();

    let edited: TitleGroupComment =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(edited.content, "Edited comment");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_edit_any_comment(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool.clone()));

    // User creates comment
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let create_body = UserCreatedTitleGroupComment {
        content: "User comment".into(),
        title_group_id: 1,
        refers_to_torrent_id: None,
        answers_to_comment_id: None,
    };

    let req = test::TestRequest::post()
        .uri("/api/title-groups/comments")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();

    let comment: TitleGroupComment =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // Staff edits it
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditTitleGroupComment,
    )
    .await;

    let edit_body = EditedTitleGroupComment {
        content: "Staff edited".into(),
        locked: false,
    };

    let req = test::TestRequest::put()
        .uri(&format!("/api/title-groups/comments/{}", comment.id))
        .insert_header(auth_header(&staff.token))
        .set_json(&edit_body)
        .to_request();

    let edited: TitleGroupComment =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(edited.content, "Staff edited");
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_comments"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_title_group_comments_returns_results(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/title-group-comments?page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<TitleGroupCommentSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 5);
    assert_eq!(response.results.len(), 5);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 10);
    let created_by = &response.results[0].created_by;
    assert_eq!(
        (created_by.id, created_by.username.as_str()),
        (1, "creator")
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_comments"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_title_group_comments_with_content_filter(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/title-group-comments?content=Beatles&page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<TitleGroupCommentSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 1);
    assert_eq!(response.results.len(), 1);
    assert!(response.results[0].content.contains("Beatles"));
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_comments"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_title_group_comments_case_insensitive(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/title-group-comments?content=AMAZING&page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<TitleGroupCommentSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Should find "RollerCoaster Tycoon is amazing!" and "The music in this game is AMAZING too!"
    assert_eq!(response.total_items, 2);
    assert_eq!(response.results.len(), 2);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_comments"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_title_group_comments_pagination(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/title-group-comments?page=1&page_size=2")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<TitleGroupCommentSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 5);
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 2);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_comments"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_title_group_comments_no_results(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/title-group-comments?content=nonexistent&page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<TitleGroupCommentSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.total_items, 0);
    assert_eq!(response.results.len(), 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_comments"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_delete_title_group_comment(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::DeleteTitleGroupComment,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/title-groups/comments/1")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Deleting it a second time fails, proving it is gone
    let req = test::TestRequest::delete()
        .uri("/api/title-groups/comments/1")
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_comments"
    ),
    migrations = "../storage/migrations"
)]
async fn test_user_without_permission_cannot_delete_title_group_comment(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .uri("/api/title-groups/comments/1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
