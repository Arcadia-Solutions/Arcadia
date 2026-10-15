pub mod common;
pub mod mocks;

use actix_http::Request;
use actix_web::{
    dev::{Service, ServiceResponse},
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        StatusCode,
    },
    test, Error,
};
use arcadia_api::config::{Config, OidcConfig};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::user::{LoginResponse, UserCreatedUserWarning},
};
use base64::{
    engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD as BASE64_URL},
    Engine,
};
use common::{auth_header, create_test_app_with_config, login_as, read_body_json_data, TestUser};
use mocks::mock_redis::MockRedisPool;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::sync::Arc;

const TEST_ISSUER: &str = "https://arcadia.example.com/api/oauth";
const TEST_CLIENT_ID: &str = "elysia-test-client";
const TEST_CLIENT_SECRET: &str = "elysia-test-client-secret";
const TEST_REDIRECT_URI: &str = "https://elysia.example.com/callback";
const TEST_CODE_VERIFIER: &str = "test-code-verifier";

#[derive(Debug, Deserialize)]
struct OAuthAuthorizeResponse {
    redirect_url: String,
}

/// The client is configured at startup, tests declare theirs by hand instead.
async fn create_test_app_with_client_and_login(
    pool: Arc<ConnectionPool>,
) -> (
    impl Service<Request, Response = ServiceResponse, Error = Error>,
    LoginResponse,
) {
    let mut config = arcadia_shared::config::load::<Config>();
    config.oidc = Some(OidcConfig {
        issuer: TEST_ISSUER.parse().unwrap(),
        client_id: TEST_CLIENT_ID.to_string(),
        client_secret: TEST_CLIENT_SECRET.to_string(),
        redirect_uri: TEST_REDIRECT_URI.to_string(),
    });
    let (service, _) = create_test_app_with_config(pool, MockRedisPool::default(), config).await;
    let user = login_as(&service, TestUser::Standard).await;
    (service, user)
}

fn authorization_query(redirect_uri: &str) -> String {
    serde_urlencoded::to_string([
        ("client_id", TEST_CLIENT_ID),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", "openid profile email"),
        ("state", "test-state"),
        (
            "code_challenge",
            &BASE64_URL.encode(Sha256::digest(TEST_CODE_VERIFIER.as_bytes())),
        ),
        ("code_challenge_method", "S256"),
    ])
    .unwrap()
}

fn query_value(url: &str, name: &str) -> Option<String> {
    serde_urlencoded::from_str::<Vec<(String, String)>>(url.split_once('?')?.1)
        .ok()?
        .into_iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
}

fn basic_client_auth(client_secret: &str) -> (actix_web::http::header::HeaderName, String) {
    (
        AUTHORIZATION,
        format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{TEST_CLIENT_ID}:{client_secret}"))
        ),
    )
}

async fn read_raw_json(
    response: ServiceResponse,
    expected_status: StatusCode,
) -> serde_json::Value {
    assert_eq!(response.status(), expected_status);
    serde_json::from_slice(&test::read_body(response).await).expect("valid JSON response")
}

/// Submits the authorization request as the logged-in user.
async fn submit_authorization<S>(service: &S, token: &str, redirect_uri: &str) -> ServiceResponse
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let request = test::TestRequest::post()
        .uri("/api/auth/oauth-authorize")
        .insert_header(auth_header(token))
        .set_json(serde_json::json!({
            "authorization_query": authorization_query(redirect_uri),
        }))
        .to_request();
    test::call_service(service, request).await
}

/// Authorizes the client as the logged-in user and returns the authorization code.
async fn authorization_code<S>(service: &S, token: &str) -> String
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let response = submit_authorization(service, token, TEST_REDIRECT_URI).await;
    assert_eq!(response.status(), StatusCode::OK);
    let authorization: OAuthAuthorizeResponse = read_body_json_data(response).await;
    assert!(authorization.redirect_url.starts_with(TEST_REDIRECT_URI));
    assert_eq!(
        query_value(&authorization.redirect_url, "state").as_deref(),
        Some("test-state")
    );
    query_value(&authorization.redirect_url, "code").expect("authorization code")
}

async fn exchange_code<S>(
    service: &S,
    code: &str,
    code_verifier: &str,
    client_secret: &str,
) -> ServiceResponse
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let body = serde_urlencoded::to_string([
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", TEST_REDIRECT_URI),
        ("code_verifier", code_verifier),
    ])
    .unwrap();
    let request = test::TestRequest::post()
        .uri("/api/oauth/token")
        .insert_header(basic_client_auth(client_secret))
        .insert_header((CONTENT_TYPE, "application/x-www-form-urlencoded"))
        .set_payload(body)
        .to_request();
    test::call_service(service, request).await
}

async fn userinfo<S>(service: &S, access_token: &str) -> ServiceResponse
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let request = test::TestRequest::get()
        .uri("/api/oauth/userinfo")
        .insert_header((AUTHORIZATION, format!("Bearer {access_token}")))
        .to_request();
    test::call_service(service, request).await
}

async fn user_status<S>(service: &S, user_id: i32) -> serde_json::Value
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let request = test::TestRequest::get()
        .uri(&format!("/api/oauth/users/{user_id}/status"))
        .insert_header(basic_client_auth(TEST_CLIENT_SECRET))
        .to_request();
    read_raw_json(test::call_service(service, request).await, StatusCode::OK).await
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_oauth_roundtrip_and_ban(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_with_client_and_login(Arc::clone(&pool)).await;

    let code = authorization_code(&service, &user.token).await;
    let token_answer = read_raw_json(
        exchange_code(&service, &code, TEST_CODE_VERIFIER, TEST_CLIENT_SECRET).await,
        StatusCode::OK,
    )
    .await;
    assert_eq!(token_answer["token_type"], "Bearer");
    let access_token = token_answer["access_token"].as_str().unwrap();

    // a reused authorization code is rejected
    let reused = read_raw_json(
        exchange_code(&service, &code, TEST_CODE_VERIFIER, TEST_CLIENT_SECRET).await,
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(reused["error"], "invalid_grant");

    let claims = read_raw_json(userinfo(&service, access_token).await, StatusCode::OK).await;
    assert_eq!(claims["sub"], "100");
    assert_eq!(claims["preferred_username"], "user_basic");
    assert!(claims["email"].is_string());
    assert_eq!(user_status(&service, 100).await["active"], true);

    pool.create_user_warning(
        100,
        &UserCreatedUserWarning {
            user_id: 100,
            expires_at: None,
            reason: "test ban".to_string(),
            ban: true,
        },
    )
    .await
    .expect("banning the user");

    assert_eq!(user_status(&service, 100).await["active"], false);
    assert_eq!(
        userinfo(&service, access_token).await.status(),
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_oauth_rejects_wrong_secret_and_pkce_mismatch(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_with_client_and_login(pool).await;

    let code = authorization_code(&service, &user.token).await;
    let wrong_secret = read_raw_json(
        exchange_code(&service, &code, TEST_CODE_VERIFIER, "wrong-secret").await,
        StatusCode::UNAUTHORIZED,
    )
    .await;
    assert_eq!(wrong_secret["error"], "invalid_client");
    let wrong_verifier = read_raw_json(
        exchange_code(&service, &code, "wrong-verifier", TEST_CLIENT_SECRET).await,
        StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(wrong_verifier["error"], "invalid_grant");
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_oauth_untrusted_redirect(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_with_client_and_login(pool).await;

    // a redirect URI other than the configured one is never redirected to
    let response = submit_authorization(
        &service,
        &user.token,
        "https://attacker.example.com/callback",
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_openid_configuration_uses_the_configured_issuer(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, _) = create_test_app_with_client_and_login(pool).await;

    let request = test::TestRequest::get()
        .uri("/api/oauth/.well-known/openid-configuration")
        .to_request();
    let configuration =
        read_raw_json(test::call_service(&service, request).await, StatusCode::OK).await;
    assert_eq!(configuration["issuer"], TEST_ISSUER);
    assert_eq!(
        configuration["token_endpoint"],
        format!("{TEST_ISSUER}/token")
    );
    assert!(configuration["authorization_endpoint"]
        .as_str()
        .unwrap()
        .ends_with("/oauth/authorize"));
}
