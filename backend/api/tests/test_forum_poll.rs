pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::forum::{ForumPoll, ForumPollHierarchy, ForumThreadEnrichedHierarchy};
use common::{TestUser, auth_header, create_test_app_and_login, login_as};
use mocks::mock_redis::MockRedisPool;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post"
    ),
    migrations = "../storage/migrations"
)]
async fn test_thread_author_creates_poll(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Thread 102 is authored by user 100 (Standard) and has no poll yet.
    let req = test::TestRequest::post()
        .uri("/api/forum/poll")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "forum_thread_id": 102,
            "question": "Best season?",
            "options": ["Summer", "Winter"]
        }))
        .to_request();
    let poll: ForumPoll =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    assert_eq!(poll.forum_thread_id, 102);
    assert_eq!(poll.question, "Best season?");

    // No blank option is stored; only the creator's options exist.
    let req = test::TestRequest::get()
        .uri("/api/forum/thread?id=102")
        .insert_header(auth_header(&user.token))
        .to_request();
    let thread: ForumThreadEnrichedHierarchy =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let poll = thread.poll.expect("poll should be attached");
    assert_eq!(poll.options.len(), 2);
    assert!(!poll.has_voted);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_forum_poll"
    ),
    migrations = "../storage/migrations"
)]
async fn test_non_author_and_duplicate_poll(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, _author) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // user 101 has create_forum_thread permission but is not the author of thread 102.
    let other = login_as(&service, TestUser::EditArtist).await;
    let req = test::TestRequest::post()
        .uri("/api/forum/poll")
        .insert_header(auth_header(&other.token))
        .set_json(json!({
            "forum_thread_id": 102,
            "question": "Should not work",
            "options": ["a", "b"]
        }))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Thread 100 already has the fixture poll -> conflict.
    let author = login_as(&service, TestUser::Standard).await;
    let req = test::TestRequest::post()
        .uri("/api/forum/poll")
        .insert_header(auth_header(&author.token))
        .set_json(json!({
            "forum_thread_id": 100,
            "question": "Another poll",
            "options": ["x", "y"]
        }))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_forum_poll"
    ),
    migrations = "../storage/migrations"
)]
async fn test_vote_hides_then_reveals_counts(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Before voting: counts hidden.
    let req = test::TestRequest::get()
        .uri("/api/forum/thread?id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let thread: ForumThreadEnrichedHierarchy =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let poll = thread.poll.expect("poll attached");
    assert!(!poll.has_voted);
    assert!(poll.blank_votes_amount.is_none());
    assert!(poll.options.iter().all(|o| o.votes_amount.is_none()));

    // Vote for option 100.
    let req = test::TestRequest::post()
        .uri("/api/forum/poll/vote")
        .insert_header(auth_header(&user.token))
        .set_json(json!({ "forum_poll_id": 100, "forum_poll_option_id": 100 }))
        .to_request();
    let voted: ForumPollHierarchy =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    assert!(voted.has_voted);
    let red = voted.options.iter().find(|o| o.id == 100).unwrap();
    assert_eq!(red.votes_amount, Some(1));
    assert_eq!(voted.blank_votes_amount, Some(0));

    // Duplicate vote rejected.
    let req = test::TestRequest::post()
        .uri("/api/forum/poll/vote")
        .insert_header(auth_header(&user.token))
        .set_json(json!({ "forum_poll_id": 100, "forum_poll_option_id": 101 }))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_forum_poll"
    ),
    migrations = "../storage/migrations"
)]
async fn test_blank_vote_and_invalid_option(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Option that does not belong to poll 100.
    let req = test::TestRequest::post()
        .uri("/api/forum/poll/vote")
        .insert_header(auth_header(&user.token))
        .set_json(json!({ "forum_poll_id": 100, "forum_poll_option_id": 999 }))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Blank vote (no option referenced).
    let req = test::TestRequest::post()
        .uri("/api/forum/poll/vote")
        .insert_header(auth_header(&user.token))
        .set_json(json!({ "forum_poll_id": 100, "forum_poll_option_id": null }))
        .to_request();
    let voted: ForumPollHierarchy =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    assert_eq!(voted.blank_votes_amount, Some(1));
    assert!(voted.options.iter().all(|o| o.votes_amount == Some(0)));

    // A second user votes for an option; counts aggregate.
    let other = login_as(&service, TestUser::EditArtist).await;
    let req = test::TestRequest::post()
        .uri("/api/forum/poll/vote")
        .insert_header(auth_header(&other.token))
        .set_json(json!({ "forum_poll_id": 100, "forum_poll_option_id": 101 }))
        .to_request();
    let voted: ForumPollHierarchy =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    assert_eq!(voted.blank_votes_amount, Some(1));
    let blue = voted.options.iter().find(|o| o.id == 101).unwrap();
    assert_eq!(blue.votes_amount, Some(1));
}
