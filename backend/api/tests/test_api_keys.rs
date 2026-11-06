use crate::{common, mocks};

use actix_http::Request;
use actix_web::{
    Error,
    dev::{Service, ServiceResponse},
    http::StatusCode,
    test,
};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::user::{APIKey, APIKeyScope, CreatedAPIKey, UserCreatedAPIKey},
    repositories::auth_repository::MAXIMUM_API_KEYS_PER_USER,
};
use common::{
    TestUser, auth_header, call_and_read_body_json, call_and_read_body_json_with_status,
    create_test_app_and_login,
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

async fn create_api_key<S>(service: &S, token: &str, scopes: Vec<APIKeyScope>) -> CreatedAPIKey
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let req = test::TestRequest::post()
        .insert_header(auth_header(token))
        .uri("/api/users/api-keys")
        .set_json(UserCreatedAPIKey {
            name: "test_key".into(),
            scopes,
        })
        .to_request();

    call_and_read_body_json_with_status::<CreatedAPIKey, _>(service, req, StatusCode::CREATED).await
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_can_create_list_and_delete_api_keys(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let created = create_api_key(&service, &user.token, vec![APIKeyScope::Torrents]).await;

    assert_eq!(created.value.len(), 40);
    assert_eq!(created.api_key.name, "test_key");
    assert_eq!(created.api_key.scopes, vec![APIKeyScope::Torrents]);
    assert!(created.value.ends_with(&created.api_key.last_four));

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/api-keys")
        .to_request();
    let api_keys = call_and_read_body_json::<Vec<APIKey>, _>(&service, req).await;

    assert_eq!(api_keys.len(), 1);
    assert_eq!(api_keys[0].id, created.api_key.id);
    assert!(api_keys[0].last_used_at.is_none());

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri(&format!("/api/users/api-keys/{}", created.api_key.id))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/api-keys")
        .to_request();
    let api_keys = call_and_read_body_json::<Vec<APIKey>, _>(&service, req).await;

    assert!(api_keys.is_empty());
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_api_key_only_reaches_the_endpoints_of_its_scopes(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let created = create_api_key(&service, &user.token, vec![APIKeyScope::User]).await;

    let req = test::TestRequest::get()
        .insert_header(("api-key", created.value.clone()))
        .uri("/api/users/me")
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let req = test::TestRequest::get()
        .insert_header(("api-key", created.value.clone()))
        .uri("/api/torrent-requests?id=1")
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // using an API key marks it as used
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/api-keys")
        .to_request();
    let api_keys = call_and_read_body_json::<Vec<APIKey>, _>(&service, req).await;
    assert!(api_keys[0].last_used_at.is_some());
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_cannot_create_more_api_keys_than_the_maximum(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    for _ in 0..MAXIMUM_API_KEYS_PER_USER {
        create_api_key(&service, &user.token, vec![APIKeyScope::User]).await;
    }

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/api-keys")
        .set_json(UserCreatedAPIKey {
            name: "one_too_many".into(),
            scopes: vec![APIKeyScope::User],
        })
        .to_request();
    let response = test::call_service(&service, req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_api_key_cannot_manage_api_keys(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let created = create_api_key(&service, &user.token, vec![APIKeyScope::User]).await;

    let req = test::TestRequest::get()
        .insert_header(("api-key", created.value.clone()))
        .uri("/api/users/api-keys")
        .to_request();
    let response = test::call_service(&service, req).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // the router percent decodes the path, so the scope check must decode it too
    let req = test::TestRequest::get()
        .insert_header(("api-key", created.value.clone()))
        .uri("/api/users/api%2Dkeys")
        .to_request();
    let response = test::call_service(&service, req).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_api_key_cannot_edit_the_account(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let created = create_api_key(&service, &user.token, vec![APIKeyScope::User]).await;

    let req = test::TestRequest::put()
        .insert_header(("api-key", created.value.clone()))
        .uri("/api/users")
        .to_request();
    let response = test::call_service(&service, req).await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_cannot_delete_the_api_key_of_another_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let created = create_api_key(&service, &user.token, vec![APIKeyScope::User]).await;
    let other_user = common::login_as(&service, TestUser::EditArtist).await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&other_user.token))
        .uri(&format!("/api/users/api-keys/{}", created.api_key.id))
        .to_request();
    let response = test::call_service(&service, req).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
