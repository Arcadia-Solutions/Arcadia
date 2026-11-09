use crate::{common, mocks};

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::artist::ArtistLite;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::forum::{ForumSubCategoryLite, ForumThreadLite};
use arcadia_storage::models::title_group::TitleGroupHierarchyLite;
use common::{auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_artist",
    "with_test_affiliated_artist",
    "with_test_forum_category",
    "with_test_forum_sub_category",
    "with_test_forum_thread",
    "with_refreshed_title_group_hierarchy_lite",
))]
async fn test_subscribe_and_unsubscribe_title_group_torrents(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Subscribe
    let req = test::TestRequest::post()
        .uri("/api/subscriptions/title-group-torrents?title_group_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Verify subscription appears
    let req = test::TestRequest::get()
        .uri("/api/subscriptions/title-group-torrents?page=1&page_size=20&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].affiliated_artists.0.len(), 1);
    assert_eq!(
        response.results[0].affiliated_artists.0[0].name,
        "The Beatles"
    );

    // Unsubscribe
    let req = test::TestRequest::delete()
        .uri("/api/subscriptions/title-group-torrents?title_group_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify subscription removed
    let req = test::TestRequest::get()
        .uri("/api/subscriptions/title-group-torrents?page=1&page_size=20&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert!(response.results.is_empty());
    assert_eq!(response.total_items, 0);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_forum_category",
    "with_test_forum_sub_category",
    "with_test_forum_thread",
))]
async fn test_subscribe_and_unsubscribe_forum_thread_posts(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Subscribe
    let req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-thread-posts?thread_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Verify subscription appears
    let req = test::TestRequest::get()
        .uri("/api/subscriptions/forum-thread-posts?page=1&page_size=20&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response: PaginatedResults<ForumThreadLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].id, 100);

    // Unsubscribe
    let req = test::TestRequest::delete()
        .uri("/api/subscriptions/forum-thread-posts?thread_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify subscription removed
    let req = test::TestRequest::get()
        .uri("/api/subscriptions/forum-thread-posts?page=1&page_size=20&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response: PaginatedResults<ForumThreadLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert!(response.results.is_empty());
    assert_eq!(response.total_items, 0);
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group", "with_test_artist",))]
async fn test_subscriptions_affiliated_artists_with_dummy_for_many_artists(pool: PgPool) {
    let pool_arc = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Insert 3 extra artists to have 4 total (artist id=1 already from fixture)
    sqlx::query(
        r#"
        INSERT INTO artists (id, name, description, pictures, created_by_id, created_at)
        VALUES
            (2, 'Artist Two', '', '{}', 1, NOW()),
            (3, 'Artist Three', '', '{}', 1, NOW()),
            (4, 'Artist Four', '', '{}', 1, NOW())
        "#,
    )
    .execute(std::borrow::Borrow::<sqlx::PgPool>::borrow(
        pool_arc.as_ref(),
    ))
    .await
    .unwrap();

    // Link all 4 artists to title_group 1 -> should trigger dummy artist
    sqlx::query(
        r#"
        INSERT INTO affiliated_artists (title_group_id, artist_id, roles, created_by_id)
        VALUES
            (1, 1, '{main}', 1),
            (1, 2, '{main}', 1),
            (1, 3, '{main}', 1),
            (1, 4, '{main}', 1)
        "#,
    )
    .execute(std::borrow::Borrow::<sqlx::PgPool>::borrow(
        pool_arc.as_ref(),
    ))
    .await
    .unwrap();

    // Subscribe to title_group 1
    sqlx::query(
        r#"
        INSERT INTO subscriptions_title_group_torrents (user_id, title_group_id)
        VALUES (100, 1)
        "#,
    )
    .execute(std::borrow::Borrow::<sqlx::PgPool>::borrow(
        pool_arc.as_ref(),
    ))
    .await
    .unwrap();

    sqlx::query("REFRESH MATERIALIZED VIEW title_group_hierarchy_lite")
        .execute(std::borrow::Borrow::<sqlx::PgPool>::borrow(
            pool_arc.as_ref(),
        ))
        .await
        .unwrap();

    let (service, user) =
        create_test_app_and_login(pool_arc, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/subscriptions/title-group-torrents?page=1&page_size=20&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.total_items, 1);
    // >2 artists: should return a single dummy artist with id=0
    let artists = &response.results[0].affiliated_artists.0;
    assert_eq!(artists.len(), 1);
    assert_eq!(artists[0].artist_id, 0);
    assert!(artists[0].name.is_empty());
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category",
))]
async fn test_subscribe_and_unsubscribe_forum_sub_category_threads(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Subscribe
    let req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-sub-category-threads?forum_sub_category_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Verify subscription appears
    let req = test::TestRequest::get()
        .uri("/api/subscriptions/forum-sub-category-threads?page=1&page_size=20&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response: PaginatedResults<ForumSubCategoryLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].id, 100);
    assert_eq!(response.results[0].name, "Test Sub Category");

    // Unsubscribe
    let req = test::TestRequest::delete()
        .uri("/api/subscriptions/forum-sub-category-threads?forum_sub_category_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify subscription removed
    let req = test::TestRequest::get()
        .uri("/api/subscriptions/forum-sub-category-threads?page=1&page_size=20&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response: PaginatedResults<ForumSubCategoryLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert!(response.results.is_empty());
    assert_eq!(response.total_items, 0);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category",
))]
async fn test_duplicate_forum_sub_category_subscription_fails(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Subscribe first time
    let req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-sub-category-threads?forum_sub_category_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Subscribe again - should fail (unique constraint violation)
    let req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-sub-category-threads?forum_sub_category_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

#[sqlx::test(fixtures("with_test_users", "with_test_artist"))]
async fn test_subscribe_and_unsubscribe_artist_title_groups(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Subscribe
    let req = test::TestRequest::post()
        .uri("/api/subscriptions/artist-title-groups?artist_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Verify subscription appears
    let req = test::TestRequest::get()
        .uri("/api/subscriptions/artist-title-groups?page=1&page_size=20&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response: PaginatedResults<ArtistLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(response.results.len(), 1);
    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].name, "The Beatles");

    // Unsubscribe
    let req = test::TestRequest::delete()
        .uri("/api/subscriptions/artist-title-groups?artist_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify subscription removed
    let req = test::TestRequest::get()
        .uri("/api/subscriptions/artist-title-groups?page=1&page_size=20&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response: PaginatedResults<ArtistLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert!(response.results.is_empty());
    assert_eq!(response.total_items, 0);
}
