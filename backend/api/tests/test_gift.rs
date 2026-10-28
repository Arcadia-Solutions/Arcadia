pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::gift::{Gift, UserCreatedGift};
use common::{
    auth_header, call_and_read_body_json_with_status, create_test_app_and_login, TestUser,
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_gift_balance"),
    migrations = "../storage/migrations"
)]
async fn test_gift_updates_sender_and_receiver_balances_and_sends_message(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Get initial balances
    let sender_before = pool.find_user_with_id(100).await.unwrap();
    let receiver_before = pool.find_user_with_id(101).await.unwrap();

    assert_eq!(sender_before.bonus_points, 1000);
    assert_eq!(sender_before.freeleech_tokens, 10);
    assert_eq!(receiver_before.bonus_points, 500);
    assert_eq!(receiver_before.freeleech_tokens, 5);

    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let gift = UserCreatedGift {
        message: "Test gift".into(),
        receiver_id: 101,
        bonus_points: 100,
        freeleech_tokens: 2,
    };

    let req = test::TestRequest::post()
        .uri("/api/gifts")
        .insert_header(auth_header(&user.token))
        .set_json(&gift)
        .to_request();

    let created_gift: Gift =
        call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    assert_eq!(created_gift.bonus_points, 100);
    assert_eq!(created_gift.freeleech_tokens, 2);
    assert_eq!(created_gift.sender_id, 100);
    assert_eq!(created_gift.receiver_id, 101);

    // Verify sender's balance was decremented
    let sender_after = pool.find_user_with_id(100).await.unwrap();
    assert_eq!(sender_after.bonus_points, 900);
    assert_eq!(sender_after.freeleech_tokens, 8);

    // Verify receiver's balance was incremented
    let receiver_after = pool.find_user_with_id(101).await.unwrap();
    assert_eq!(receiver_after.bonus_points, 600);
    assert_eq!(receiver_after.freeleech_tokens, 7);

    // Verify receiver got a message (from user ID 1)
    let search_query = arcadia_storage::models::conversation::ConversationSearchQuery {
        search_term: None,
        search_titles_only: false,
        page: 1,
        page_size: 50,
        user_id: None,
        order_by_column:
            arcadia_storage::models::conversation::ConversationSearchOrderByColumn::LastMessage,
        order_by_direction: arcadia_storage::models::common::OrderByDirection::Desc,
        all_conversations: false,
    };
    let conversations = pool
        .search_conversations(101, &search_query, false)
        .await
        .unwrap();

    assert!(!conversations.results.is_empty());

    let gift_conversation = conversations
        .results
        .iter()
        .find(|c| c.subject == "You received a gift!")
        .expect("Gift notification conversation not found");

    assert_eq!(gift_conversation.sender_id, 1);
    assert_eq!(gift_conversation.receiver_id, 101);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_gift_balance"),
    migrations = "../storage/migrations"
)]
async fn test_concurrent_gifts_can_not_spend_more_than_the_balance(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let gift = UserCreatedGift {
        message: "Test gift".into(),
        receiver_id: 101,
        bonus_points: 600,
        freeleech_tokens: 0,
    };

    let (first, second) =
        futures::join!(pool.create_gift(&gift, 100), pool.create_gift(&gift, 100));
    assert!(first.is_ok() != second.is_ok());

    let sender_after = pool.find_user_with_id(100).await.unwrap();
    assert_eq!(sender_after.bonus_points, 400);
}
