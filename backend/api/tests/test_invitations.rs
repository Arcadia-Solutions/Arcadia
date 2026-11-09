use crate::{common, mocks};
use actix_web::http::Method;
use common::authenticated_request;
use serde_json::{Value, json};

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::invitation::InvitationHierarchy;
use common::{TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

fn search_uri(show_foreign_invitations: bool) -> String {
    format!(
        "/api/invitations?page=1&page_size=50\
         &order_by_column=created_at&order_by_direction=desc\
         &show_foreign_invitations={show_foreign_invitations}"
    )
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_invitations"),
    migrations = "../storage/migrations"
)]
async fn test_search_sent_invitations_without_permission_only_sees_own_invitations(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&search_uri(false))
        .insert_header(auth_header(&user.token))
        .to_request();
    let results: PaginatedResults<InvitationHierarchy> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.total_items, 1);
    assert_eq!(results.results.len(), 1);
    assert_eq!(
        results.results[0].receiver_email,
        "own_invite@testdomain.com"
    );
    assert!(results.results[0].sender.is_none());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_invitations"),
    migrations = "../storage/migrations"
)]
async fn test_search_sent_invitations_with_show_foreign_invitations_without_permission_is_forbidden(
    pool: PgPool,
) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&search_uri(true))
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_invitations"),
    migrations = "../storage/migrations"
)]
async fn test_search_sent_invitations_with_permission_sees_foreign_invitations(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::ViewForeignInvitations,
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&search_uri(true))
        .insert_header(auth_header(&user.token))
        .to_request();
    let results: PaginatedResults<InvitationHierarchy> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.total_items, 2);
    let foreign_invitation = results
        .results
        .iter()
        .find(|invitation| invitation.receiver_email == "own_invite@testdomain.com")
        .expect("own invitation, sent by user_basic, should be visible");
    let sender = foreign_invitation
        .sender
        .as_ref()
        .expect("sender should be hydrated when show_foreign_invitations is honored");
    assert_eq!(sender.username, "user_basic");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_invitations"),
    migrations = "../storage/migrations"
)]
async fn test_search_sent_invitations_with_permission_but_flag_unset_only_sees_own(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::ViewForeignInvitations,
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&search_uri(false))
        .insert_header(auth_header(&user.token))
        .to_request();
    let results: PaginatedResults<InvitationHierarchy> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.total_items, 1);
    assert_eq!(
        results.results[0].receiver_email,
        "foreign_invite@testdomain.com"
    );
}

fn invitation() -> Option<Value> {
    Some(json!({ "message": "hello", "receiver_email": "friend@testdomain.com" }))
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_invitation_available"),
    migrations = "../storage/migrations"
)]
async fn test_inviting_an_applicant_requires_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    // inviting an applicant accepts their application, which is a staff decision
    let mut invitation = invitation().unwrap();
    invitation["user_application_id"] = json!(1);
    let request = authenticated_request(
        Method::POST,
        "/api/invitations",
        &user.token,
        Some(invitation),
    );
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(pool.find_user_with_id(100).await.unwrap().invitations, 1);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_invitation_requires_an_available_invitation(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let request =
        authenticated_request(Method::POST, "/api/invitations", &user.token, invitation());
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_invitation_available"),
    migrations = "../storage/migrations"
)]
async fn test_invitation_consumes_an_available_invitation(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let request =
        authenticated_request(Method::POST, "/api/invitations", &user.token, invitation());
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(pool.find_user_with_id(100).await.unwrap().invitations, 0);

    let request =
        authenticated_request(Method::POST, "/api/invitations", &user.token, invitation());
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
