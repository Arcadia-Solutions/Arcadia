use crate::{common, mocks};

use actix_web::{http::StatusCode, test::TestRequest};
use arcadia_api::handlers::users::reset_passkey::ResetPasskeyResponse;
use arcadia_storage::{connection_pool::ConnectionPool, models::user::UserCreatedUserWarning};
use common::{TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"))]
async fn test_reset_passkey_replaces_the_stored_passkey(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let previous_passkey = pool
        .find_user_by_username("user_basic")
        .await
        .unwrap()
        .passkey;

    let req = TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/passkey")
        .to_request();
    let response = call_and_read_body_json::<ResetPasskeyResponse, _>(&service, req).await;

    assert_eq!(response.passkey.len(), 32);
    assert_ne!(response.passkey, previous_passkey);
    assert_eq!(
        pool.find_user_by_username("user_basic")
            .await
            .unwrap()
            .passkey,
        response.passkey
    );
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_banned_user_cannot_reset_passkey(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let previous_passkey = pool
        .find_user_by_username("user_basic")
        .await
        .unwrap()
        .passkey;
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
    .unwrap();

    let req = TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/passkey")
        .to_request();
    let response = actix_web::test::call_service(&service, req).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        pool.find_user_by_username("user_basic")
            .await
            .unwrap()
            .passkey,
        previous_passkey
    );
}
