pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::{
    forum::{ForumPost, ForumThread, UserCreatedForumPost, UserCreatedForumThread},
    notification::{NotificationCounts, Notifications},
    staff_pm::{StaffPm, StaffPmMessage, UserCreatedStaffPm, UserCreatedStaffPmMessage},
    title_group_comment::{TitleGroupComment, UserCreatedTitleGroupComment},
    torrent_request_comment::TorrentRequestComment,
};
use common::{auth_header, create_test_app, create_test_app_and_login, TestUser};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

// Title Group Comment Notifications

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_subscriber_receives_notification_on_new_title_group_comment(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A (Standard) will create comments
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B (EditTitleGroupComment) will subscribe and receive notifications
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    // User B subscribes to title group 1
    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/title-group-comments?title_group_id=1")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A creates a comment on title group 1
    let create_body = UserCreatedTitleGroupComment {
        content: "Test comment for notification".into(),
        title_group_id: 1,
        refers_to_torrent_id: None,
        answers_to_comment_id: None,
    };
    let req = test::TestRequest::post()
        .uri("/api/title-groups/comments")
        .insert_header(auth_header(&user_a.token))
        .set_json(&create_body)
        .to_request();
    let _comment: TitleGroupComment =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User B checks notifications
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.title_group_comments.len(), 1);
    assert_eq!(notifications.title_group_comments[0].title_group_id, 1);
    assert!(!notifications.title_group_comments[0].read_status);

    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.title_group_comments, 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_comment_creator_does_not_receive_own_notification(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User subscribes to title group 1
    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/title-group-comments?title_group_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Same user creates a comment
    let create_body = UserCreatedTitleGroupComment {
        content: "My own comment".into(),
        title_group_id: 1,
        refers_to_torrent_id: None,
        answers_to_comment_id: None,
    };
    let req = test::TestRequest::post()
        .uri("/api/title-groups/comments")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();
    let _comment: TitleGroupComment =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User should NOT receive notification for their own comment
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;

    assert_eq!(notifications.title_group_comments.len(), 0);

    // Verify counter stays at 0
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, me_req).await;
    assert_eq!(counts.title_group_comments, 0);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_no_duplicate_unread_title_group_notifications(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A creates comments
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B subscribes
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/title-group-comments?title_group_id=1")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A creates two comments
    for i in 1..=2 {
        let create_body = UserCreatedTitleGroupComment {
            content: format!("Comment {}", i),
            title_group_id: 1,
            refers_to_torrent_id: None,
            answers_to_comment_id: None,
        };
        let req = test::TestRequest::post()
            .uri("/api/title-groups/comments")
            .insert_header(auth_header(&user_a.token))
            .set_json(&create_body)
            .to_request();
        let _: TitleGroupComment =
            common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    }

    // User B should only have 1 unread notification (no duplicates)
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.title_group_comments.len(), 1);

    // Verify counter is 1 (not 2)
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.title_group_comments, 1);
}

// Forum Thread Post Notifications

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread"
    ),
    migrations = "../storage/migrations"
)]
async fn test_subscriber_receives_notification_on_new_forum_post(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A creates posts
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B subscribes and receives notifications
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    // User B subscribes to forum thread 100
    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-thread-posts?thread_id=100")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A creates a post in thread 100
    let create_body = UserCreatedForumPost {
        content: "Test post for notification".into(),
        forum_thread_id: 100,
    };
    let req = test::TestRequest::post()
        .uri("/api/forum/post")
        .insert_header(auth_header(&user_a.token))
        .set_json(&create_body)
        .to_request();
    let _post: ForumPost =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User B checks notifications
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.forum_thread_posts.len(), 1);
    assert_eq!(notifications.forum_thread_posts[0].forum_thread_id, 100);
    assert!(!notifications.forum_thread_posts[0].read_status);

    // Verify counter
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.forum_thread_posts, 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread"
    ),
    migrations = "../storage/migrations"
)]
async fn test_post_creator_does_not_receive_own_notification(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User subscribes to thread 100
    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-thread-posts?thread_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Same user creates a post
    let create_body = UserCreatedForumPost {
        content: "My own post".into(),
        forum_thread_id: 100,
    };
    let req = test::TestRequest::post()
        .uri("/api/forum/post")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();
    let _post: ForumPost =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User should NOT receive notification for their own post
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;

    assert_eq!(notifications.forum_thread_posts.len(), 0);

    // Verify counter stays at 0
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, me_req).await;
    assert_eq!(counts.forum_thread_posts, 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread"
    ),
    migrations = "../storage/migrations"
)]
async fn test_no_duplicate_unread_forum_notifications(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A creates posts
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B subscribes
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-thread-posts?thread_id=100")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A creates two posts
    for i in 1..=2 {
        let create_body = UserCreatedForumPost {
            content: format!("Post {}", i),
            forum_thread_id: 100,
        };
        let req = test::TestRequest::post()
            .uri("/api/forum/post")
            .insert_header(auth_header(&user_a.token))
            .set_json(&create_body)
            .to_request();
        let _: ForumPost =
            common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    }

    // User B should only have 1 unread notification (no duplicates)
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.forum_thread_posts.len(), 1);

    // Verify counter is 1 (not 2)
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.forum_thread_posts, 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_include_read_filter_title_group_notifications(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A creates comments
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B subscribes
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/title-group-comments?title_group_id=1")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A creates a comment
    let create_body = UserCreatedTitleGroupComment {
        content: "Comment to mark as read".into(),
        title_group_id: 1,
        refers_to_torrent_id: None,
        answers_to_comment_id: None,
    };
    let req = test::TestRequest::post()
        .uri("/api/title-groups/comments")
        .insert_header(auth_header(&user_a.token))
        .set_json(&create_body)
        .to_request();
    let _: TitleGroupComment =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // With include_read=false, should see 1 notification
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;
    assert_eq!(notifications.title_group_comments.len(), 1);

    // Verify counter
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.title_group_comments, 1);

    // With include_read=true, should also see 1 notification (same result, just includes read ones too)
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=true")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;
    assert_eq!(notifications.title_group_comments.len(), 1);
}

// Conversation Notifications

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_conversation_counter_increments_for_receiver(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A (Standard, id=100) will create a conversation
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B (EditTitleGroupComment, id=103) will receive the conversation
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    // User B should have 0 unread conversations initially
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.conversations, 0);

    // User A creates a conversation with User B (id=103)
    let req = test::TestRequest::post()
        .uri("/api/conversations")
        .insert_header(auth_header(&user_a.token))
        .set_json(serde_json::json!({
            "subject": "Test conversation",
            "receiver_id": 103,
            "first_message": {
                "conversation_id": 0,
                "content": "Hello, this is a test message"
            }
        }))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User B should now have 1 unread conversation
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.conversations, 1);
}

// Staff PM Notifications

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_receives_notification_on_new_staff_pm(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A (Standard) creates a staff PM
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B (Staff with ReadStaffPm permission, id=138) should receive notification
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_staff_pm",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    // User A creates a staff PM
    let create_body = UserCreatedStaffPm {
        subject: "Test Staff PM".into(),
        first_message: UserCreatedStaffPmMessage {
            staff_pm_id: 0,
            content: "This is a test staff PM message".into(),
        },
    };
    let req = test::TestRequest::post()
        .uri("/api/staff-pms")
        .insert_header(auth_header(&user_a.token))
        .set_json(&create_body)
        .to_request();
    let _staff_pm: StaffPm =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User B (staff) checks notifications
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.staff_pm_messages.len(), 1);
    assert_eq!(
        notifications.staff_pm_messages[0].staff_pm_subject,
        "Test Staff PM"
    );
    assert!(!notifications.staff_pm_messages[0].read_status);

    // Verify counter
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.staff_pm_messages, 1);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_creator_receives_notification_on_staff_reply(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A (Standard) creates a staff PM
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B (Staff) will reply
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_staff_pm",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    // User A creates a staff PM
    let create_body = UserCreatedStaffPm {
        subject: "Test Staff PM for reply".into(),
        first_message: UserCreatedStaffPmMessage {
            staff_pm_id: 0,
            content: "Initial message".into(),
        },
    };
    let req = test::TestRequest::post()
        .uri("/api/staff-pms")
        .insert_header(auth_header(&user_a.token))
        .set_json(&create_body)
        .to_request();
    let staff_pm: StaffPm =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // Clear user A's notifications (they created the PM so shouldn't have any)
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_a.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;
    assert_eq!(notifications.staff_pm_messages.len(), 0);

    // User B (staff) replies to the staff PM
    let reply_body = UserCreatedStaffPmMessage {
        staff_pm_id: staff_pm.id,
        content: "Staff reply".into(),
    };
    let req = test::TestRequest::post()
        .uri("/api/staff-pms/messages")
        .insert_header(auth_header(&user_b.token))
        .set_json(&reply_body)
        .to_request();
    let _reply: StaffPmMessage =
        common::call_and_read_body_json_with_status(&service_b, req, StatusCode::CREATED).await;

    // User A (creator) should now have a notification
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_a.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;

    assert_eq!(notifications.staff_pm_messages.len(), 1);
    assert_eq!(notifications.staff_pm_messages[0].staff_pm_id, staff_pm.id);

    // Verify counter
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_a.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, me_req).await;
    assert_eq!(counts.staff_pm_messages, 1);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_message_creator_does_not_receive_own_staff_pm_notification(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A (Standard) creates a staff PM
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User A creates a staff PM
    let create_body = UserCreatedStaffPm {
        subject: "Test Staff PM".into(),
        first_message: UserCreatedStaffPmMessage {
            staff_pm_id: 0,
            content: "My own message".into(),
        },
    };
    let req = test::TestRequest::post()
        .uri("/api/staff-pms")
        .insert_header(auth_header(&user_a.token))
        .set_json(&create_body)
        .to_request();
    let _staff_pm: StaffPm =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User A should NOT receive notification for their own message
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_a.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;

    assert_eq!(notifications.staff_pm_messages.len(), 0);

    // Verify counter stays at 0
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_a.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, me_req).await;
    assert_eq!(counts.staff_pm_messages, 0);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_notifications_marked_as_read_when_staff_pm_resolved(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A (Standard) creates a staff PM
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B (Staff) will resolve the PM
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_staff_pm",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    // User A creates a staff PM
    let create_body = UserCreatedStaffPm {
        subject: "Staff PM to resolve".into(),
        first_message: UserCreatedStaffPmMessage {
            staff_pm_id: 0,
            content: "Please help".into(),
        },
    };
    let req = test::TestRequest::post()
        .uri("/api/staff-pms")
        .insert_header(auth_header(&user_a.token))
        .set_json(&create_body)
        .to_request();
    let staff_pm: StaffPm =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User B (staff) should have unread notification
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;
    assert_eq!(notifications.staff_pm_messages.len(), 1);

    // Verify counter before resolve
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.staff_pm_messages, 1);

    // User B resolves the staff PM
    let resolve_req = test::TestRequest::put()
        .uri(&format!("/api/staff-pms/{}/resolve", staff_pm.id))
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, resolve_req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // User B should have no unread notifications now
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;
    assert_eq!(notifications.staff_pm_messages.len(), 0);

    // Verify counter after resolve
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.staff_pm_messages, 0);

    // But with include_read=true, should still see it
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=true")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;
    assert_eq!(notifications.staff_pm_messages.len(), 1);
    assert!(notifications.staff_pm_messages[0].read_status);
}

// Torrent Request Comment Notifications

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_request"
    ),
    migrations = "../storage/migrations"
)]
async fn test_subscriber_receives_notification_on_new_torrent_request_comment(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A (Standard) will create comments
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B (EditTitleGroupComment) will subscribe and receive notifications
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    // User B subscribes to torrent request 1
    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/torrent-request-comments?torrent_request_id=1")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A creates a comment on torrent request 1
    let req = test::TestRequest::post()
        .uri("/api/torrent-requests/comment")
        .insert_header(auth_header(&user_a.token))
        .set_json(serde_json::json!({
            "torrent_request_id": 1,
            "content": "Test comment for notification"
        }))
        .to_request();
    let _comment: TorrentRequestComment =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User B checks notifications
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.torrent_request_comments.len(), 1);
    assert_eq!(
        notifications.torrent_request_comments[0].torrent_request_id,
        1
    );
    assert!(!notifications.torrent_request_comments[0].read_status);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_request"
    ),
    migrations = "../storage/migrations"
)]
async fn test_comment_creator_does_not_receive_own_torrent_request_notification(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User subscribes to torrent request 1
    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/torrent-request-comments?torrent_request_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Same user creates a comment
    let req = test::TestRequest::post()
        .uri("/api/torrent-requests/comment")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({
            "torrent_request_id": 1,
            "content": "My own comment"
        }))
        .to_request();
    let _comment: TorrentRequestComment =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User should NOT receive notification for their own comment
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;

    assert_eq!(notifications.torrent_request_comments.len(), 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_request"
    ),
    migrations = "../storage/migrations"
)]
async fn test_no_duplicate_unread_torrent_request_notifications(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A creates comments
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B subscribes
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/torrent-request-comments?torrent_request_id=1")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A creates two comments
    for i in 1..=2 {
        let req = test::TestRequest::post()
            .uri("/api/torrent-requests/comment")
            .insert_header(auth_header(&user_a.token))
            .set_json(serde_json::json!({
                "torrent_request_id": 1,
                "content": format!("Comment {}", i)
            }))
            .to_request();
        let _: TorrentRequestComment =
            common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    }

    // User B should only have 1 unread notification (no duplicates)
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.torrent_request_comments.len(), 1);
}

// Unread Announcements

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_unread_announcements_counts_unseen_threads_in_subcategory_1(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // The initdb creates one thread (id=1) in subcategory 1 (Announcements).
    // A fresh user has never viewed it, so announcements should be 1.
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, me_req).await;
    assert_eq!(counts.announcements, 1);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_unread_announcements_decreases_after_viewing_thread(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // View the announcement thread's posts (thread id=1), which creates a read marker
    let req = test::TestRequest::get()
        .uri("/api/forum/thread/posts?thread_id=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // After viewing, the announcement should be counted as read
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, me_req).await;
    assert_eq!(counts.announcements, 0);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_unread_announcements_stays_read_after_new_post_in_thread(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A views the announcement
    let (service_a, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/forum/thread/posts?thread_id=1&page_size=10")
        .insert_header(auth_header(&user_a.token))
        .to_request();
    let resp = test::call_service(&service_a, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // User B posts a new reply in the announcement thread
    let (service_b, user_b) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::EditArtist)
            .await;

    let post_body = UserCreatedForumPost {
        content: "New reply in announcement".into(),
        forum_thread_id: 1,
    };
    let req = test::TestRequest::post()
        .uri("/api/forum/post")
        .insert_header(auth_header(&user_b.token))
        .set_json(&post_body)
        .to_request();
    let _: ForumPost =
        common::call_and_read_body_json_with_status(&service_b, req, StatusCode::CREATED).await;

    // User A's announcements should still be 0 because the thread was seen once
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_a.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_a, me_req).await;
    assert_eq!(counts.announcements, 0);
}

// Forum Sub-Category Thread Notifications

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_subscriber_receives_notification_on_new_thread_in_sub_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // User A creates threads
    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User B subscribes and receives notifications
    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    // User B subscribes to sub-category 100
    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-sub-category-threads?forum_sub_category_id=100")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A creates a thread in sub-category 100
    let create_body = UserCreatedForumThread {
        forum_sub_category_id: 100,
        name: "New thread for notification test".into(),
        first_post: UserCreatedForumPost {
            content: "First post content".into(),
            forum_thread_id: 0,
        },
    };
    let req = test::TestRequest::post()
        .uri("/api/forum/thread")
        .insert_header(auth_header(&user_a.token))
        .set_json(&create_body)
        .to_request();
    let _thread: ForumThread =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // User B checks notifications
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.forum_sub_category_threads.len(), 1);
    assert_eq!(
        notifications.forum_sub_category_threads[0].forum_sub_category_id,
        100
    );
    assert!(!notifications.forum_sub_category_threads[0].read_status);

    // Verify counter
    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.forum_sub_category_threads, 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_thread_creator_does_not_receive_own_sub_category_notification(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    // User subscribes to sub-category 100
    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-sub-category-threads?forum_sub_category_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Same user creates a thread
    let create_body = UserCreatedForumThread {
        forum_sub_category_id: 100,
        name: "My own thread".into(),
        first_post: UserCreatedForumPost {
            content: "First post".into(),
            forum_thread_id: 0,
        },
    };
    let req = test::TestRequest::post()
        .uri("/api/forum/thread")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();
    let _thread: ForumThread =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    // Should not have sub-category notification for own thread
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;

    assert_eq!(notifications.forum_sub_category_threads.len(), 0);

    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, me_req).await;
    assert_eq!(counts.forum_sub_category_threads, 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category"
    ),
    migrations = "../storage/migrations"
)]
async fn test_viewing_thread_clears_sub_category_new_thread_notifications(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let mock_redis = MockRedisPool::default();
    let service_b = create_test_app(pool.clone(), mock_redis).await;
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();
    let user_b: arcadia_storage::models::user::LoginResponse =
        common::call_and_read_body_json(&service_b, login_req).await;

    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/forum-sub-category-threads?forum_sub_category_id=100")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let create_body = UserCreatedForumThread {
        forum_sub_category_id: 100,
        name: "Thread to trigger notification".into(),
        first_post: UserCreatedForumPost {
            content: "First post content".into(),
            forum_thread_id: 0,
        },
    };
    let req = test::TestRequest::post()
        .uri("/api/forum/thread")
        .insert_header(auth_header(&user_a.token))
        .set_json(&create_body)
        .to_request();
    let thread: ForumThread =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.forum_sub_category_threads, 1);

    let view_req = test::TestRequest::get()
        .uri(&format!("/api/forum/thread?id={}", thread.id))
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, view_req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let me_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, me_req).await;
    assert_eq!(counts.forum_sub_category_threads, 0);

    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=true")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;
    assert_eq!(notifications.forum_sub_category_threads.len(), 1);
    assert!(notifications.forum_sub_category_threads[0].read_status);
}

// Artist Title Group Notifications

fn title_group_body_with_artists(name: &str, artist_ids: &[i64]) -> serde_json::Value {
    let affiliated_artists: Vec<serde_json::Value> = artist_ids
        .iter()
        .map(|artist_id| {
            serde_json::json!({
                "title_group_id": 0,
                "artist_id": artist_id,
                "roles": ["main"],
                "nickname": null
            })
        })
        .collect();

    serde_json::json!({
        "name": name,
        "name_aliases": [],
        "description": "A title group created for notification tests",
        "original_language": null,
        "country_from": null,
        "covers": [],
        "external_links": [],
        "trailers": [],
        "category": null,
        "content_type": "music",
        "tags": ["rock"],
        "tagline": null,
        "platform": null,
        "original_release_date": null,
        "original_release_date_only_year_known": false,
        "affiliated_artists": affiliated_artists,
        "series_id": null,
        "screenshots": [],
        "master_group_id": null
    })
}

async fn login_second_user(
    service: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
) -> arcadia_storage::models::user::LoginResponse {
    let login_req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": "user_edit_tgc",
            "password": "test_password",
            "remember_me": true
        }))
        .to_request();

    common::call_and_read_body_json(service, login_req).await
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artist"),
    migrations = "../storage/migrations"
)]
async fn test_subscriber_receives_notification_on_new_title_group_of_artist(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let service_b = create_test_app(pool.clone(), MockRedisPool::default()).await;
    let user_b = login_second_user(&service_b).await;

    // User B subscribes to artist 1
    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/artist-title-groups?artist_id=1")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A creates a title group affiliated to artist 1
    let req = test::TestRequest::post()
        .uri("/api/title-groups")
        .insert_header(auth_header(&user_a.token))
        .set_json(title_group_body_with_artists("Abbey Road", &[1]))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User B is notified
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.artist_title_groups.len(), 1);
    assert_eq!(notifications.artist_title_groups[0].artist_id, 1);
    assert_eq!(
        notifications.artist_title_groups[0].artist_name,
        "The Beatles"
    );
    assert_eq!(
        notifications.artist_title_groups[0].title_group_name,
        "Abbey Road"
    );
    assert!(!notifications.artist_title_groups[0].read_status);

    let counts_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, counts_req).await;
    assert_eq!(counts.artist_title_groups, 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artist", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_subscriber_receives_notification_on_artist_affiliated_to_existing_title_group(
    pool: PgPool,
) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user_a) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditTitleGroup,
    )
    .await;
    let service_b = create_test_app(pool.clone(), MockRedisPool::default()).await;
    let user_b = login_second_user(&service_b).await;

    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/artist-title-groups?artist_id=1")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User A affiliates artist 1 to the existing title group 1
    let req = test::TestRequest::post()
        .uri("/api/affiliated-artists")
        .insert_header(auth_header(&user_a.token))
        .set_json(serde_json::json!([{
            "title_group_id": 1,
            "artist_id": 1,
            "roles": ["main"],
            "nickname": null
        }]))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.artist_title_groups.len(), 1);
    assert_eq!(notifications.artist_title_groups[0].title_group_id, 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artists_for_search"),
    migrations = "../storage/migrations"
)]
async fn test_only_one_notification_when_several_subscribed_artists_share_a_title_group(
    pool: PgPool,
) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user_a) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let service_b = create_test_app(pool.clone(), MockRedisPool::default()).await;
    let user_b = login_second_user(&service_b).await;

    for artist_id in [1, 2] {
        let sub_req = test::TestRequest::post()
            .uri(&format!(
                "/api/subscriptions/artist-title-groups?artist_id={artist_id}"
            ))
            .insert_header(auth_header(&user_b.token))
            .to_request();
        let resp = test::call_service(&service_b, sub_req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    let req = test::TestRequest::post()
        .uri("/api/title-groups")
        .insert_header(auth_header(&user_a.token))
        .set_json(title_group_body_with_artists("A Collaboration", &[1, 2]))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;

    assert_eq!(notifications.artist_title_groups.len(), 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artist"),
    migrations = "../storage/migrations"
)]
async fn test_affiliation_creator_does_not_receive_own_artist_notification(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/artist-title-groups?artist_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let req = test::TestRequest::post()
        .uri("/api/title-groups")
        .insert_header(auth_header(&user.token))
        .set_json(title_group_body_with_artists("Let It Be", &[1]))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;

    assert!(notifications.artist_title_groups.is_empty());
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_title_group_torrent_notification",
    ),
    migrations = "../storage/migrations"
)]
async fn test_title_group_torrent_notifications_are_listed(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;

    assert_eq!(notifications.title_group_torrents.len(), 1);
    assert_eq!(notifications.title_group_torrents[0].torrent_id, 1);
    assert_eq!(notifications.title_group_torrents[0].title_group_id, 1);

    let counts_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, counts_req).await;
    assert_eq!(counts.title_group_torrents, 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_title_group_torrent_notification",
    ),
    migrations = "../storage/migrations"
)]
async fn test_visiting_title_group_marks_torrent_notifications_as_read(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let title_group_req = test::TestRequest::get()
        .uri("/api/title-groups?id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, title_group_req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let counts_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, counts_req).await;
    assert_eq!(counts.title_group_torrents, 0);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artist", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_visiting_title_group_marks_artist_notifications_as_read(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service, user_a) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditTitleGroup,
    )
    .await;
    let service_b = create_test_app(pool.clone(), MockRedisPool::default()).await;
    let user_b = login_second_user(&service_b).await;

    let sub_req = test::TestRequest::post()
        .uri("/api/subscriptions/artist-title-groups?artist_id=1")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, sub_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let req = test::TestRequest::post()
        .uri("/api/affiliated-artists")
        .insert_header(auth_header(&user_a.token))
        .set_json(serde_json::json!([{
            "title_group_id": 1,
            "artist_id": 1,
            "roles": ["main"],
            "nickname": null
        }]))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // User B opens the title group the notification points to
    let title_group_req = test::TestRequest::get()
        .uri("/api/title-groups?id=1")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let resp = test::call_service(&service_b, title_group_req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service_b, notif_req).await;
    assert!(notifications.artist_title_groups.is_empty());

    let counts_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user_b.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service_b, counts_req).await;
    assert_eq!(counts.artist_title_groups, 0);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_notification_streams_are_limited_per_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;
    let stream_request = || {
        test::TestRequest::get()
            .uri(&format!("/api/notifications/stream?token={}", user.token))
            .to_request()
    };

    // the responses are kept alive so that their streams stay open
    let mut open_streams = Vec::new();
    for _ in 0..5 {
        let resp = test::call_service(&service, stream_request()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        open_streams.push(resp);
    }

    let resp = test::call_service(&service, stream_request()).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    // closing a stream frees its slot
    open_streams.pop();
    let resp = test::call_service(&service, stream_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
