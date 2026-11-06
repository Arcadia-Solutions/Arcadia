use crate::{common, mocks};

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::conversation::{
    ConversationHierarchy, ConversationSearchResult, MassMessageResult,
};
use arcadia_storage::models::user::{Login, LoginResponse};
use common::auth_header;
use common::create_test_app_and_login;
use common::{call_and_read_body_json, call_and_read_body_json_with_status};
use mocks::mock_redis::MockRedisPool;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_users_for_search"),
    migrations = "../storage/migrations"
)]
async fn test_mass_conversation_without_permission_is_forbidden(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .uri("/api/conversations/mass")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "username": "alice",
            "subject": "Hello",
            "message": "Hi everyone",
        }))
        .to_request();

    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_users_for_search"),
    migrations = "../storage/migrations"
)]
async fn test_mass_conversation_sends_to_every_matching_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SendMassPm).await;

    // alice_smith and alice_wonder match, so the message must reach both of them.
    let req = test::TestRequest::post()
        .uri("/api/conversations/mass")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "username": "alice",
            "subject": "Hello",
            "message": "Hi everyone",
        }))
        .to_request();

    let result: MassMessageResult =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(result.messages_sent, 2);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_users_for_search"),
    migrations = "../storage/migrations"
)]
async fn test_mass_conversation_replaces_username_placeholder(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, sender) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SendMassPm).await;

    let req = test::TestRequest::post()
        .uri("/api/conversations/mass")
        .insert_header(auth_header(&sender.token))
        .set_json(json!({
            "username": "alice_smith",
            "subject": "Hello",
            "message": "Hi {username}, welcome!",
        }))
        .to_request();
    let _: MassMessageResult =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Log in as the recipient and read the message they received.
    let login = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(Login {
            username: "alice_smith".into(),
            password: "test_password".into(),
            remember_me: true,
        })
        .to_request();
    let recipient: LoginResponse = call_and_read_body_json(&service, login).await;

    let req = test::TestRequest::get()
        .uri("/api/search/conversations?page=1&page_size=50&search_titles_only=true&order_by_column=last_message&order_by_direction=desc&all_conversations=false")
        .insert_header(auth_header(&recipient.token))
        .to_request();
    let conversations: PaginatedResults<ConversationSearchResult> =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let conversation_id = conversations.results[0].conversation_id;

    let req = test::TestRequest::get()
        .uri(&format!("/api/conversations?id={conversation_id}"))
        .insert_header(auth_header(&recipient.token))
        .to_request();
    let conversation: ConversationHierarchy = call_and_read_body_json(&service, req).await;

    // {username} became a BBCode link to the recipient's own profile.
    let content = &conversation.messages[0].content;
    assert!(
        content.contains("alice_smith"),
        "expected the username, got: {content}"
    );
    assert!(
        !content.contains("{username}"),
        "placeholder should be gone: {content}"
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_users_with_permissions"),
    migrations = "../storage/migrations"
)]
async fn test_mass_conversation_only_reaches_the_users_matching_the_permissions(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SendMassPmWithPermissions,
    )
    .await;

    // perm_download and perm_both may download torrents, perm_upload may not.
    let req = test::TestRequest::post()
        .uri("/api/conversations/mass")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "username": "perm_",
            "permissions": ["download_torrent"],
            "permissions_match": "any",
            "subject": "Hello",
            "message": "Hi everyone",
        }))
        .to_request();

    let result: MassMessageResult =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(result.messages_sent, 2);
}
