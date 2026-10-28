pub mod common;
pub mod mocks;

use actix_http::Request;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::http::StatusCode;
use actix_web::test;
use actix_web::Error;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::conversation::{
    ConversationHierarchy, UserCreatedConversationMessage,
};
use chrono::{DateTime, TimeZone, Utc};
use common::{auth_header, call_and_read_body_json, create_test_app_and_login, login_as, TestUser};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_conversation"),
    migrations = "../storage/migrations"
)]
async fn test_cannot_send_message_to_locked_conversation(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let message = UserCreatedConversationMessage {
        conversation_id: 100,
        content: "This should fail".into(),
    };

    let req = test::TestRequest::post()
        .uri("/api/conversations/messages")
        .insert_header(auth_header(&user.token))
        .set_json(&message)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_conversation"),
    migrations = "../storage/migrations"
)]
async fn test_third_party_cannot_send_message_to_conversation(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, third_party) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditSeries).await;

    let req = test::TestRequest::post()
        .uri("/api/conversations/messages")
        .insert_header(auth_header(&third_party.token))
        .set_json(UserCreatedConversationMessage {
            conversation_id: 101,
            content: "injected".into(),
        })
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert!(!resp.status().is_success());
}

const FIXTURE_LAST_SEEN_AT: &str = "2020-01-01T00:00:00Z";

async fn read_conversation<S>(
    service: &S,
    token: &str,
    conversation_id: i64,
) -> ConversationHierarchy
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
{
    let req = test::TestRequest::get()
        .uri(&format!("/api/conversations?id={conversation_id}"))
        .insert_header(auth_header(token))
        .to_request();
    call_and_read_body_json(service, req).await
}

fn fixture_anchor() -> DateTime<Utc> {
    FIXTURE_LAST_SEEN_AT.parse().unwrap()
}

/// Any timestamp produced by NOW() during the test must be far above this.
fn just_now_threshold() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_conversation"),
    migrations = "../storage/migrations"
)]
async fn test_sender_reading_conversation_marks_only_sender_as_read(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, sender) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;
    let observer = login_as(&service, TestUser::ReadAllConversationsThirdParty).await;

    let _ = read_conversation(&service, &sender.token, 101).await;

    let after = read_conversation(&service, &observer.token, 101).await;
    assert!(
        after.sender_last_seen_at > just_now_threshold(),
        "sender_last_seen_at should have been bumped to NOW()"
    );
    assert!(
        after.receiver_last_seen_at.is_none(),
        "receiver_last_seen_at should remain untouched"
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_conversation"),
    migrations = "../storage/migrations"
)]
async fn test_receiver_reading_conversation_marks_only_receiver_as_read(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, receiver) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditArtist).await;
    let observer = login_as(&service, TestUser::ReadAllConversationsThirdParty).await;

    let _ = read_conversation(&service, &receiver.token, 101).await;

    let after = read_conversation(&service, &observer.token, 101).await;
    assert_eq!(
        after.sender_last_seen_at,
        fixture_anchor(),
        "sender_last_seen_at should remain untouched"
    );
    let receiver_seen = after
        .receiver_last_seen_at
        .expect("receiver_last_seen_at should have been set");
    assert!(
        receiver_seen > just_now_threshold(),
        "receiver_last_seen_at should have been bumped to NOW()"
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_conversation"),
    migrations = "../storage/migrations"
)]
async fn test_third_party_without_permission_cannot_read_conversation(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, third_party) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditSeries).await;
    let observer = login_as(&service, TestUser::ReadAllConversationsThirdParty).await;

    let req = test::TestRequest::get()
        .uri("/api/conversations?id=101")
        .insert_header(auth_header(&third_party.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert!(
        !resp.status().is_success(),
        "non-member without read_all should not be able to read the conversation"
    );

    let after = read_conversation(&service, &observer.token, 101).await;
    assert_eq!(after.sender_last_seen_at, fixture_anchor());
    assert!(after.receiver_last_seen_at.is_none());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_conversation"),
    migrations = "../storage/migrations"
)]
async fn test_third_party_with_read_all_permission_does_not_change_read_status(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, observer) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ReadAllConversationsThirdParty,
    )
    .await;

    let response = read_conversation(&service, &observer.token, 101).await;
    assert_eq!(response.sender_last_seen_at, fixture_anchor());
    assert!(response.receiver_last_seen_at.is_none());

    let after = read_conversation(&service, &observer.token, 101).await;
    assert_eq!(after.sender_last_seen_at, fixture_anchor());
    assert!(after.receiver_last_seen_at.is_none());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_conversation"),
    migrations = "../storage/migrations"
)]
async fn test_conversation_member_reading_conversation_and_read_all_permission_still_marks_as_read(
    pool: PgPool,
) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, member) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ReadAllConversationsMember,
    )
    .await;
    let observer = login_as(&service, TestUser::ReadAllConversationsThirdParty).await;

    let _ = read_conversation(&service, &member.token, 102).await;

    let after = read_conversation(&service, &observer.token, 102).await;
    assert!(
        after.sender_last_seen_at > just_now_threshold(),
        "sender_last_seen_at should be bumped even when the reader has read_all_conversations"
    );
    assert!(
        after.receiver_last_seen_at.is_none(),
        "receiver_last_seen_at should remain untouched"
    );
}
