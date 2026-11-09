use crate::{common, mocks};
use actix_web::http::{Method, StatusCode};
use arcadia_api::config::Config;
use common::{authenticated_request, create_test_app_with_config, login_as, spawn_http_server};
use serde_json::{Value, json};

use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use common::{Profile, TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_get_me_for_regular_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/me")
        .to_request();

    let profile = call_and_read_body_json::<Profile, _>(&service, req).await;

    assert_eq!(profile.user.username, "user_basic");
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_moderation_requires_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    for (method, uri, body) in [
        (Method::DELETE, "/api/users/101/warnings", None),
        (
            Method::PUT,
            "/api/users/101/custom-title",
            Some(json!({ "custom_title": "custom" })),
        ),
    ] {
        let request = authenticated_request(method.clone(), uri, &user.token, body);
        let response = test::call_service(&service, request).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{method} {uri}");
    }
}

fn config_with_irc_enabled(ergo_api_url: &str) -> Config {
    let mut config = arcadia_shared::config::load::<Config>();
    config.ergo.api_url = Some(ergo_api_url.to_string());
    config.ergo.api_bearer_token = Some("test_ergo_api_token".to_string());
    config
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_irc_account_creation_and_password_reset(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (ergo_url, ergo_requests) =
        spawn_http_server("200 OK", json!({ "success": true }).to_string());
    let (service, _) = create_test_app_with_config(
        Arc::clone(&pool),
        MockRedisPool::default(),
        config_with_irc_enabled(&ergo_url),
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;
    let irc_request =
        |method: Method| authenticated_request(method, "/api/users/irc", &user.token, None);

    // there is no account to reset the password of yet
    let response = test::call_service(&service, irc_request(Method::PUT)).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = test::call_service(&service, irc_request(Method::POST)).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let created: Value = test::read_body_json(response).await;
    let created_password = created["data"]["irc_password"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(ergo_requests.recv().unwrap().contains(&created_password));
    assert_eq!(
        pool.find_user_with_id(100)
            .await
            .unwrap()
            .irc_password
            .as_deref(),
        Some(created_password.as_str())
    );

    let response = test::call_service(&service, irc_request(Method::POST)).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let response = test::call_service(&service, irc_request(Method::PUT)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let reset: Value = test::read_body_json(response).await;
    let reset_password = reset["data"]["irc_password"].as_str().unwrap();
    assert_ne!(reset_password, created_password);
    assert_eq!(
        pool.find_user_with_id(100)
            .await
            .unwrap()
            .irc_password
            .as_deref(),
        Some(reset_password)
    );
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_irc_account_creation_fails_when_ergo_refuses_it(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (ergo_url, _) = spawn_http_server(
        "200 OK",
        json!({ "success": false, "error": "taken", "errorCode": "ACCOUNT_EXISTS" }).to_string(),
    );
    let (service, _) = create_test_app_with_config(
        Arc::clone(&pool),
        MockRedisPool::default(),
        config_with_irc_enabled(&ergo_url),
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let request = authenticated_request(Method::POST, "/api/users/irc", &user.token, None);
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        pool.find_user_with_id(100)
            .await
            .unwrap()
            .irc_password
            .is_none()
    );
}
