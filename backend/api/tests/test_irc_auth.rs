use crate::common::create_test_app_with_config;
use crate::mocks::mock_redis::MockRedisPool;
use actix_web::{
    http::{
        StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue},
    },
    test::{TestRequest, call_service, read_body_json},
};
use arcadia_api::config::Config;
use arcadia_storage::connection_pool::ConnectionPool;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

const CALLBACK_TOKEN: &str = "test_ergo_callback_token";

// Answers whether ergo would accept the login, given the callback token and credentials it sent.
async fn irc_login(
    pool: PgPool,
    callback_token: Option<&str>,
    account_name: &str,
    passphrase: &str,
) -> Value {
    let mut config = arcadia_shared::config::load::<Config>();
    config.ergo.auth_callback_token = Some(CALLBACK_TOKEN.to_string());
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app_with_config(pool, MockRedisPool::default(), config).await;

    let mut request = TestRequest::post()
        .uri("/api/auth/irc")
        .set_json(json!({ "accountName": account_name, "passphrase": passphrase }));
    if let Some(callback_token) = callback_token {
        request = request.insert_header((AUTHORIZATION, format!("Bearer {callback_token}")));
    }
    let response = call_service(&service, request.to_request()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE),
        Some(&HeaderValue::from_static("application/json"))
    );
    read_body_json(response).await
}

#[sqlx::test(fixtures("with_test_users", "with_test_irc_password"))]
async fn test_irc_auth_accepts_the_raw_password(pool: PgPool) {
    // ergo reads the answer as is, so it must not be wrapped in the side effects envelope
    let answer = irc_login(
        pool,
        Some(CALLBACK_TOKEN),
        "user_basic",
        "irc_secret_password",
    )
    .await;
    assert_eq!(answer, json!({ "success": true }));
}

#[sqlx::test(fixtures("with_test_users", "with_test_irc_password"))]
async fn test_irc_auth_accepts_the_password_prefixed_with_the_account_name(pool: PgPool) {
    let answer = irc_login(
        pool,
        Some(CALLBACK_TOKEN),
        "user_basic",
        "user_basic:irc_secret_password",
    )
    .await;
    assert_eq!(answer["success"], true);
}

#[sqlx::test(fixtures("with_test_users", "with_test_irc_password"))]
async fn test_irc_auth_rejects_a_wrong_password(pool: PgPool) {
    let answer = irc_login(pool, Some(CALLBACK_TOKEN), "user_basic", "wrong_password").await;
    assert_eq!(answer["success"], false);
}

#[sqlx::test(fixtures("with_test_users", "with_test_irc_password"))]
async fn test_irc_auth_rejects_the_password_prefixed_with_another_account_name(pool: PgPool) {
    let answer = irc_login(
        pool,
        Some(CALLBACK_TOKEN),
        "user_basic",
        "user_edit_art:irc_secret_password",
    )
    .await;
    assert_eq!(answer["success"], false);
}

#[sqlx::test(fixtures("with_test_users", "with_test_irc_password"))]
async fn test_irc_auth_rejects_a_missing_callback_token(pool: PgPool) {
    let answer = irc_login(pool, None, "user_basic", "irc_secret_password").await;
    assert_eq!(answer["success"], false);
}

#[sqlx::test(fixtures("with_test_users", "with_test_irc_password"))]
async fn test_irc_auth_rejects_a_wrong_callback_token(pool: PgPool) {
    let answer = irc_login(
        pool,
        Some("wrong_token"),
        "user_basic",
        "irc_secret_password",
    )
    .await;
    assert_eq!(answer["success"], false);
}

#[sqlx::test(fixtures("with_test_users", "with_test_irc_password"))]
async fn test_irc_auth_rejects_an_unknown_account(pool: PgPool) {
    let answer = irc_login(pool, Some(CALLBACK_TOKEN), "nobody", "irc_secret_password").await;
    assert_eq!(answer["success"], false);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_irc_auth_rejects_an_account_without_irc_password(pool: PgPool) {
    // an empty passphrase must not match the missing password
    let answer = irc_login(pool, Some(CALLBACK_TOKEN), "user_basic", "").await;
    assert_eq!(answer["success"], false);
}

#[sqlx::test(fixtures("with_test_users", "with_test_irc_password"))]
async fn test_irc_auth_fails_when_irc_is_disabled(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let mut config = arcadia_shared::config::load::<Config>();
    config.ergo.auth_callback_token = None;
    let service = create_test_app_with_config(pool, MockRedisPool::default(), config).await;

    let request = TestRequest::post()
        .uri("/api/auth/irc")
        .insert_header((AUTHORIZATION, format!("Bearer {CALLBACK_TOKEN}")))
        .set_json(json!({ "accountName": "user_basic", "passphrase": "irc_secret_password" }))
        .to_request();
    let response = call_service(&service, request).await;
    assert!(!response.status().is_success());
}
