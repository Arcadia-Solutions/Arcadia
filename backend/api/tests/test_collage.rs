pub mod common;
pub mod mocks;
use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::collage::{Collage, EditedCollage, UserCreatedCollageEntry};
use arcadia_storage::models::notification::{NotificationCounts, Notifications};
use common::{auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_collage"),
    migrations = "../storage/migrations"
)]
async fn test_owner_can_edit_own_collage(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::put()
        .uri("/api/collages")
        .insert_header(auth_header(&user.token))
        .set_json(EditedCollage {
            id: 1,
            name: "Updated Name".into(),
            cover: None,
            description: "Updated".into(),
            tags: vec!["updated".into()],
            category: arcadia_storage::models::collage::CollageCategory::Personal,
        })
        .to_request();

    let resp: Collage =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(resp.name, "Updated Name");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_collage"),
    migrations = "../storage/migrations"
)]
async fn test_user_cannot_edit_others_collage(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::put()
        .uri("/api/collages")
        .insert_header(auth_header(&user.token))
        .set_json(EditedCollage {
            id: 2, // owned by user 101
            name: "Hacked".into(),
            cover: None,
            description: "Hacked".into(),
            tags: vec![],
            category: arcadia_storage::models::collage::CollageCategory::Personal,
        })
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_collage"),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_edit_any_collage(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditCollage).await;

    let req = test::TestRequest::put()
        .uri("/api/collages")
        .insert_header(auth_header(&user.token))
        .set_json(EditedCollage {
            id: 1,
            name: "Staff Edit".into(),
            cover: None,
            description: "Edited by staff".into(),
            tags: vec![],
            category: arcadia_storage::models::collage::CollageCategory::Theme,
        })
        .to_request();

    let resp: Collage =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(resp.name, "Staff Edit");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_collage"),
    migrations = "../storage/migrations"
)]
async fn test_user_cannot_delete_collage(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .uri("/api/collages?collage_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_collage"),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_delete_collage(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteCollage,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/collages?collage_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify deletion
    assert!(pool.find_collage(1).await.is_err());
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_collage",
        "with_test_collage_entry"
    ),
    migrations = "../storage/migrations"
)]
async fn test_user_cannot_delete_collage_entry(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .uri("/api/collages/entries?collage_id=1&title_group_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_collage",
        "with_test_collage_entry"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_delete_collage_entry(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteCollageEntry,
    )
    .await;

    assert!(pool.find_collage_entry(1, 1).await.is_ok());

    let req = test::TestRequest::delete()
        .uri("/api/collages/entries?collage_id=1&title_group_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the entry was actually deleted
    assert!(pool.find_collage_entry(1, 1).await.is_err());
}
#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group", "with_test_collage"),
    migrations = "../storage/migrations"
)]
async fn test_subscriber_receives_notification_on_new_collage_entry(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let (subscriber_service, subscriber) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditCollage,
    )
    .await;
    let subscribe_req = test::TestRequest::post()
        .uri("/api/subscriptions/collages?collage_id=1")
        .insert_header(auth_header(&subscriber.token))
        .to_request();
    let resp = test::call_service(&subscriber_service, subscribe_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let create_body = vec![UserCreatedCollageEntry {
        collage_id: 1,
        title_group_id: 1,
        note: None,
    }];
    let req = test::TestRequest::post()
        .uri("/api/collages/entries")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&subscriber.token))
        .to_request();
    let notifications: Notifications =
        common::call_and_read_body_json(&subscriber_service, notif_req).await;
    assert_eq!(notifications.collages.len(), 1);
    assert_eq!(notifications.collages[0].collage_id, 1);
    assert_eq!(notifications.collages[0].title_group_id, 1);
    assert!(!notifications.collages[0].read_status);
    let counts_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&subscriber.token))
        .to_request();
    let counts: NotificationCounts =
        common::call_and_read_body_json(&subscriber_service, counts_req).await;
    assert_eq!(counts.collages, 1);
}
#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group", "with_test_collage"),
    migrations = "../storage/migrations"
)]
async fn test_subscriber_receives_multiple_notifications_for_same_collage(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let (subscriber_service, subscriber) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditCollage,
    )
    .await;
    let subscribe_req = test::TestRequest::post()
        .uri("/api/subscriptions/collages?collage_id=1")
        .insert_header(auth_header(&subscriber.token))
        .to_request();
    let resp = test::call_service(&subscriber_service, subscribe_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let create_body = vec![UserCreatedCollageEntry {
        collage_id: 1,
        title_group_id: 1,
        note: None,
    }];
    let req = test::TestRequest::post()
        .uri("/api/collages/entries")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let create_body = vec![UserCreatedCollageEntry {
        collage_id: 1,
        title_group_id: 2,
        note: None,
    }];
    let req = test::TestRequest::post()
        .uri("/api/collages/entries")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&subscriber.token))
        .to_request();
    let notifications: Notifications =
        common::call_and_read_body_json(&subscriber_service, notif_req).await;
    assert_eq!(notifications.collages.len(), 2);
    assert!(notifications.collages.iter().any(|notification| {
        notification.collage_id == 1
            && notification.title_group_id == 1
            && !notification.read_status
    }));
    assert!(notifications.collages.iter().any(|notification| {
        notification.collage_id == 1
            && notification.title_group_id == 2
            && !notification.read_status
    }));
    let counts_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&subscriber.token))
        .to_request();
    let counts: NotificationCounts =
        common::call_and_read_body_json(&subscriber_service, counts_req).await;
    assert_eq!(counts.collages, 2);
}
#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group", "with_test_collage"),
    migrations = "../storage/migrations"
)]
async fn test_collage_entry_creator_does_not_receive_own_notification(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let subscribe_req = test::TestRequest::post()
        .uri("/api/subscriptions/collages?collage_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, subscribe_req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let create_body = vec![UserCreatedCollageEntry {
        collage_id: 1,
        title_group_id: 1,
        note: None,
    }];
    let req = test::TestRequest::post()
        .uri("/api/collages/entries")
        .insert_header(auth_header(&user.token))
        .set_json(&create_body)
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let notif_req = test::TestRequest::get()
        .uri("/api/notifications?include_read=false")
        .insert_header(auth_header(&user.token))
        .to_request();
    let notifications: Notifications = common::call_and_read_body_json(&service, notif_req).await;
    assert!(notifications.collages.is_empty());
    let counts_req = test::TestRequest::get()
        .uri("/api/notifications/counts")
        .insert_header(auth_header(&user.token))
        .to_request();
    let counts: NotificationCounts = common::call_and_read_body_json(&service, counts_req).await;
    assert_eq!(counts.collages, 0);
}
#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group", "with_test_collage"),
    migrations = "../storage/migrations"
)]
async fn test_only_owner_or_collage_editor_can_add_entries_to_personal_and_staff_picks_collages(
    pool: PgPool,
) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (standard_service, standard_user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let (editor_service, editor) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditCollage).await;

    // collages 2, 4 and 5 belong to another user: a theme, a personal and a staff picks collage
    for (service, user, collage_id, expected_status) in [
        (&standard_service, &standard_user, 2, StatusCode::CREATED),
        (&standard_service, &standard_user, 4, StatusCode::FORBIDDEN),
        (&standard_service, &standard_user, 5, StatusCode::FORBIDDEN),
        (&editor_service, &editor, 4, StatusCode::CREATED),
        (&editor_service, &editor, 5, StatusCode::CREATED),
    ] {
        let req = test::TestRequest::post()
            .uri("/api/collages/entries")
            .insert_header(auth_header(&user.token))
            .set_json(vec![UserCreatedCollageEntry {
                collage_id,
                title_group_id: 1,
                note: None,
            }])
            .to_request();
        let resp = test::call_service(service, req).await;
        assert_eq!(resp.status(), expected_status, "collage {collage_id}");
    }
}
