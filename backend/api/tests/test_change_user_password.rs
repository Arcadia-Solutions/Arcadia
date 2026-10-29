pub mod common;
pub mod mocks;

use actix_web::{
    http::StatusCode,
    test::{call_service, TestRequest},
};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::user::{EditedUser, GeneratedPasswordResetToken, Login, LoginResponse},
};
use common::{auth_header, create_test_app_and_login, login_as, read_body_json_data, TestUser};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

const NEW_PASSWORD: &str = "BrandNewPassword123";

fn login_request(username: &str, password: &str) -> actix_http::Request {
    TestRequest::post()
        .uri("/api/auth/login")
        .set_json(Login {
            username: username.into(),
            password: password.into(),
            remember_me: true,
        })
        .to_request()
}

fn me_request(token: &str) -> actix_http::Request {
    TestRequest::get()
        .insert_header(auth_header(token))
        .uri("/api/users/me")
        .to_request()
}

fn token_of(reset_url: &str) -> String {
    reset_url
        .split_once("token=")
        .expect("the reset url contains a token")
        .1
        .to_string()
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_self_change_password_success(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/password")
        .set_json(serde_json::json!({
            "current_password": "test_password",
            "new_password": NEW_PASSWORD,
            "new_password_verify": NEW_PASSWORD,
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let new_tokens = read_body_json_data::<LoginResponse, _>(resp).await;

    let resp = call_service(&service, login_request("user_basic", NEW_PASSWORD)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = call_service(&service, login_request("user_basic", "test_password")).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // the sessions opened with the old password are closed
    let resp = call_service(&service, me_request(&user.token)).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // but the user does not have to log in again
    let resp = call_service(&service, me_request(&new_tokens.token)).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_self_change_password_revokes_the_reset_token(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, admin) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::GenerateResetPasswordToken,
    )
    .await;

    let req = TestRequest::post()
        .insert_header(auth_header(&admin.token))
        .uri("/api/users/100/password-reset-token")
        .to_request();

    let resp = call_service(&service, req).await;
    let generated = read_body_json_data::<GeneratedPasswordResetToken, _>(resp).await;
    let token = token_of(&generated.reset_url);

    let user = login_as(&service, TestUser::Standard).await;
    let req = TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/password")
        .set_json(serde_json::json!({
            "current_password": "test_password",
            "new_password": NEW_PASSWORD,
            "new_password_verify": NEW_PASSWORD,
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // the link handed out before the password change is not usable anymore
    let req = TestRequest::post()
        .uri("/api/auth/reset-password")
        .set_json(serde_json::json!({
            "token": token,
            "new_password": "AnotherNewPassword123",
            "new_password_verify": "AnotherNewPassword123",
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_self_change_password_wrong_current_password(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/password")
        .set_json(serde_json::json!({
            "current_password": "wrong_password",
            "new_password": NEW_PASSWORD,
            "new_password_verify": NEW_PASSWORD,
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let resp = call_service(&service, login_request("user_basic", "test_password")).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_self_change_password_mismatched_verification(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/password")
        .set_json(serde_json::json!({
            "current_password": "test_password",
            "new_password": NEW_PASSWORD,
            "new_password_verify": "DifferentPassword123",
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_create_password_reset_token_without_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/118/password-reset-token")
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_reset_password_with_generated_token(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, admin) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::GenerateResetPasswordToken,
    )
    .await;

    let req = TestRequest::post()
        .insert_header(auth_header(&admin.token))
        .uri("/api/users/100/password-reset-token")
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let generated = read_body_json_data::<GeneratedPasswordResetToken, _>(resp).await;
    let token = token_of(&generated.reset_url);

    let reset_request = || {
        TestRequest::post()
            .uri("/api/auth/reset-password")
            .set_json(serde_json::json!({
                "token": token,
                "new_password": NEW_PASSWORD,
                "new_password_verify": NEW_PASSWORD,
            }))
            .to_request()
    };

    // the reset page is reachable without being authenticated
    let resp = call_service(&service, reset_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = call_service(&service, login_request("user_basic", NEW_PASSWORD)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // the token can only be used once
    let resp = call_service(&service, reset_request()).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_reset_password_with_invalid_token(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, _) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = TestRequest::post()
        .uri("/api/auth/reset-password")
        .set_json(serde_json::json!({
            "token": "definitely_not_a_valid_token",
            "new_password": NEW_PASSWORD,
            "new_password_verify": NEW_PASSWORD,
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let resp = call_service(&service, login_request("user_basic", "test_password")).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_reset_password_with_weak_password_keeps_the_token_usable(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, admin) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::GenerateResetPasswordToken,
    )
    .await;

    let req = TestRequest::post()
        .insert_header(auth_header(&admin.token))
        .uri("/api/users/100/password-reset-token")
        .to_request();

    let resp = call_service(&service, req).await;
    let generated = read_body_json_data::<GeneratedPasswordResetToken, _>(resp).await;
    let token = token_of(&generated.reset_url);

    let req = TestRequest::post()
        .uri("/api/auth/reset-password")
        .set_json(serde_json::json!({
            "token": token,
            "new_password": "weak",
            "new_password_verify": "weak",
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let req = TestRequest::post()
        .uri("/api/auth/reset-password")
        .set_json(serde_json::json!({
            "token": token,
            "new_password": NEW_PASSWORD,
            "new_password_verify": NEW_PASSWORD,
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_changing_email_requires_current_password(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    for (current_password, expected_status) in [
        (None, StatusCode::BAD_REQUEST),
        (Some("wrong_password"), StatusCode::BAD_REQUEST),
        (Some("test_password"), StatusCode::OK),
    ] {
        let req = TestRequest::put()
            .uri("/api/users")
            .insert_header(auth_header(&user.token))
            .set_json(EditedUser {
                avatar: None,
                email: "new_email@testdomain.com".into(),
                description: String::new(),
                current_password: current_password.map(Into::into),
            })
            .to_request();
        assert_eq!(call_service(&service, req).await.status(), expected_status);
    }
}
