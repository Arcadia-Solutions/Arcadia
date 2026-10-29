pub mod common;
pub mod mocks;

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::artist::UserCreatedArtist;
use arcadia_storage::models::title_group::{create_default_title_group, UserCreatedTitleGroup};
use arcadia_storage::models::user::EditedUser;
use common::auth_header;
use common::create_test_app_and_login;
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_approved_image_hosts"),
    migrations = "../storage/migrations"
)]
async fn test_approved_image_host_allows_valid_urls(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Create artist with approved image host
    let req_body = vec![UserCreatedArtist {
        name: "Test Artist".into(),
        aliases: vec![],
        description: "A test artist".into(),
        pictures: vec!["https://i.imgur.com/test.jpg".into()],
        external_links: vec![],
    }];

    let req = test::TestRequest::post()
        .uri("/api/artists")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[sqlx::test(
    fixtures("with_test_users", "with_approved_image_hosts"),
    migrations = "../storage/migrations"
)]
async fn test_approved_image_host_rejects_unapproved_urls(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Create artist with unapproved image host
    let req_body = vec![UserCreatedArtist {
        name: "Test Artist".into(),
        aliases: vec![],
        description: "A test artist".into(),
        pictures: vec!["https://evil.example.com/malware.jpg".into()],
        external_links: vec![],
    }];

    let req = test::TestRequest::post()
        .uri("/api/artists")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(
    fixtures("with_test_users", "with_approved_image_hosts"),
    migrations = "../storage/migrations"
)]
async fn test_user_avatar_rejects_unapproved_host(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req_body = EditedUser {
        avatar: Some("https://evil.example.com/avatar.jpg".into()),
        email: "test_user@testdomain.com".into(),
        description: "Updated description".into(),
        current_password: None,
    };

    let req = test::TestRequest::put()
        .uri("/api/users")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(
    fixtures("with_test_users", "with_approved_image_hosts"),
    migrations = "../storage/migrations"
)]
async fn test_user_avatar_allows_approved_host(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req_body = EditedUser {
        avatar: Some("https://i.imgur.com/avatar.jpg".into()),
        email: "test_user@testdomain.com".into(),
        description: "Updated description".into(),
        current_password: None,
    };

    let req = test::TestRequest::put()
        .uri("/api/users")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_empty_approved_image_hosts_allows_all_urls(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // When approved_image_hosts is empty, all URLs should be allowed
    let req_body = vec![UserCreatedArtist {
        name: "Test Artist".into(),
        aliases: vec![],
        description: "A test artist".into(),
        pictures: vec!["https://any-random-host.com/image.jpg".into()],
        external_links: vec![],
    }];

    let req = test::TestRequest::post()
        .uri("/api/artists")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_title_group_rejects_non_http_trailer(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req_body = UserCreatedTitleGroup {
        trailers: vec!["javascript:alert(document.domain)".into()],
        ..create_default_title_group()
    };

    let req = test::TestRequest::post()
        .uri("/api/title-groups")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
