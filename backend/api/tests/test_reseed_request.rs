use crate::{common, mocks};

use std::sync::Arc;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{connection_pool::ConnectionPool, models::notification::NotificationEvent};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{TestUser, auth_header};

const REQUESTER_ID: i32 = 197;
const PAST_SEEDER_ID: i32 = 100;
const SNATCHER_ID: i32 = 101;

/// Torrent 1 has no seeders. User 100 seeded it 100h ago, user 101 only completed it,
/// and the requester (197) also seeded it 100h ago.
async fn set_up_dead_torrent(pool: &PgPool) {
    sqlx::query("UPDATE torrents SET seeders = 0, leechers = 0 WHERE id = 1")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO torrent_activities (torrent_id, user_id, first_seen_seeding_at, last_seen_seeding_at, completed_at)
         VALUES (1, 100, NOW() - INTERVAL '200 hours', NOW() - INTERVAL '100 hours', NULL),
                (1, 101, NULL, NULL, NOW() - INTERVAL '150 hours'),
                (1, 197, NOW() - INTERVAL '200 hours', NOW() - INTERVAL '100 hours', NULL)",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn reseed_rows(pool: &PgPool, user_id: i32, unread_only: bool) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications_reseed_requests
         WHERE user_id = $1 AND ($2 = FALSE OR read_status = FALSE)",
    )
    .bind(user_id)
    .bind(unread_only)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn total_rows(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM notifications_reseed_requests")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn reseed_request_notifies_past_seeders_and_snatchers(pool: PgPool) {
    set_up_dead_torrent(&pool).await;
    let (service, mut notification_receiver) = common::create_test_app_with_notifications(
        Arc::new(ConnectionPool::with_pg_pool(pool.clone())),
        MockRedisPool::default(),
    )
    .await;
    let user = common::login_as(&service, TestUser::RequestReseed).await;

    let req = test::TestRequest::post()
        .uri("/api/torrents/1/reseed-request")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    match notification_receiver.try_recv() {
        Ok(NotificationEvent::ReseedRequest { mut user_ids }) => {
            user_ids.sort();
            assert_eq!(user_ids, vec![PAST_SEEDER_ID, SNATCHER_ID]);
        }
        other => panic!("expected a ReseedRequest notification event, got {other:?}"),
    }

    assert_eq!(reseed_rows(&pool, PAST_SEEDER_ID, false).await, 1);
    assert_eq!(reseed_rows(&pool, SNATCHER_ID, false).await, 1);
    assert_eq!(reseed_rows(&pool, REQUESTER_ID, false).await, 0);
    assert_eq!(total_rows(&pool).await, 2);

    let db = ConnectionPool::with_pg_pool(pool.clone());
    let notifications = db
        .find_all_notifications(PAST_SEEDER_ID, false)
        .await
        .unwrap();
    assert_eq!(notifications.reseed_requests.len(), 1);
    assert_eq!(notifications.reseed_requests[0].torrent_id, 1);
    assert_eq!(notifications.reseed_requests[0].requested_by.len(), 1);
    assert_eq!(
        notifications.reseed_requests[0].requested_by[0].id,
        REQUESTER_ID
    );
    let counts = db.find_notification_counts(PAST_SEEDER_ID).await.unwrap();
    assert_eq!(counts.reseed_requests, 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn reseed_requests_from_several_users_aggregate_into_one_notification(pool: PgPool) {
    set_up_dead_torrent(&pool).await;
    let db = ConnectionPool::with_pg_pool(pool.clone());

    // two different users request a reseed for the same torrent
    db.request_reseed(1, REQUESTER_ID, 72).await.unwrap();
    db.request_reseed(1, PAST_SEEDER_ID, 72).await.unwrap();

    // the snatcher received a request from both requesters, as a single notification line
    let notifications = db.find_all_notifications(SNATCHER_ID, false).await.unwrap();
    assert_eq!(notifications.reseed_requests.len(), 1);
    assert_eq!(notifications.reseed_requests[0].torrent_id, 1);
    let mut requester_ids: Vec<i32> = notifications.reseed_requests[0]
        .requested_by
        .iter()
        .map(|user| user.id)
        .collect();
    requester_ids.sort();
    assert_eq!(requester_ids, vec![PAST_SEEDER_ID, REQUESTER_ID]);

    // and it still counts as a single notification
    let counts = db.find_notification_counts(SNATCHER_ID).await.unwrap();
    assert_eq!(counts.reseed_requests, 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn reseed_request_rejected_when_requester_was_only_contributor(pool: PgPool) {
    // torrent 1 is dead and the requester (197) is the only past seeder/snatcher
    sqlx::query("UPDATE torrents SET seeders = 0, leechers = 0 WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO torrent_activities (torrent_id, user_id, first_seen_seeding_at, last_seen_seeding_at, completed_at)
         VALUES (1, 197, NOW() - INTERVAL '200 hours', NOW() - INTERVAL '100 hours', NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let (service, user) = common::create_test_app_and_login(
        Arc::new(ConnectionPool::with_pg_pool(pool.clone())),
        MockRedisPool::default(),
        TestUser::RequestReseed,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/torrents/1/reseed-request")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(total_rows(&pool).await, 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn reseed_request_rejected_when_same_user_already_requested(pool: PgPool) {
    set_up_dead_torrent(&pool).await;
    let service = common::create_test_app(
        Arc::new(ConnectionPool::with_pg_pool(pool.clone())),
        MockRedisPool::default(),
    )
    .await;
    let user = common::login_as(&service, TestUser::RequestReseed).await;

    let send = || {
        test::TestRequest::post()
            .uri("/api/torrents/1/reseed-request")
            .insert_header(auth_header(&user.token))
            .to_request()
    };

    let first = test::call_service(&service, send()).await;
    assert_eq!(first.status(), StatusCode::OK);

    let second = test::call_service(&service, send()).await;
    assert_eq!(second.status(), StatusCode::CONFLICT);

    // the rejected second request must not create any additional rows
    assert_eq!(total_rows(&pool).await, 2);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn title_group_hierarchy_exposes_last_seeded_at_only_for_dead_torrents(pool: PgPool) {
    set_up_dead_torrent(&pool).await;
    let db = ConnectionPool::with_pg_pool(pool.clone());

    let dead = db
        .find_title_group_hierarchy(1, REQUESTER_ID)
        .await
        .unwrap();
    assert!(dead.edition_groups[0].torrents[0].last_seeded_at.is_some());

    sqlx::query("UPDATE torrents SET seeders = 2 WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();
    let alive = db
        .find_title_group_hierarchy(1, REQUESTER_ID)
        .await
        .unwrap();
    assert!(alive.edition_groups[0].torrents[0].last_seeded_at.is_none());
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn reseed_request_rejected_when_torrent_has_seeders(pool: PgPool) {
    set_up_dead_torrent(&pool).await;
    sqlx::query("UPDATE torrents SET seeders = 2 WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();
    let (service, user) = common::create_test_app_and_login(
        Arc::new(ConnectionPool::with_pg_pool(pool.clone())),
        MockRedisPool::default(),
        TestUser::RequestReseed,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/torrents/1/reseed-request")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(total_rows(&pool).await, 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn reseed_request_rejected_when_not_dead_long_enough(pool: PgPool) {
    set_up_dead_torrent(&pool).await;
    sqlx::query(
        "UPDATE torrent_activities SET last_seen_seeding_at = NOW() - INTERVAL '1 hour'
         WHERE torrent_id = 1",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (service, user) = common::create_test_app_and_login(
        Arc::new(ConnectionPool::with_pg_pool(pool.clone())),
        MockRedisPool::default(),
        TestUser::RequestReseed,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/torrents/1/reseed-request")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(total_rows(&pool).await, 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn reseed_request_rejected_when_never_seeded(pool: PgPool) {
    // no torrent_activities rows at all, the torrent has no seeders
    let (service, user) = common::create_test_app_and_login(
        Arc::new(ConnectionPool::with_pg_pool(pool.clone())),
        MockRedisPool::default(),
        TestUser::RequestReseed,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/torrents/1/reseed-request")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert_eq!(total_rows(&pool).await, 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn reseed_request_forbidden_without_permission(pool: PgPool) {
    set_up_dead_torrent(&pool).await;
    let (service, user) = common::create_test_app_and_login(
        Arc::new(ConnectionPool::with_pg_pool(pool.clone())),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/torrents/1/reseed-request")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    assert_eq!(total_rows(&pool).await, 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn auto_clear_deletes_only_healthy_torrents(pool: PgPool) {
    // torrent 1 is healthy (seeders >= 1, no leechers), torrent 11 has leechers,
    // torrent 12 has no seeders
    for (id, seeders, leechers) in [(1, 1, 0), (11, 1, 3), (12, 0, 0)] {
        if id != 1 {
            sqlx::query(
                "INSERT INTO torrents (id, edition_group_id, created_by_id, info_hash, info_dict, languages,
                     release_name, release_group, description, file_amount_per_type, uploaded_as_anonymous,
                     file_list, mediainfo, trumpable, staff_checked, container, size, duration, audio_codec,
                     audio_bitrate, audio_bitrate_sampling, audio_channels, video_codec, features,
                     subtitle_languages, video_resolution)
                 SELECT $1, edition_group_id, created_by_id,
                     overlay(info_hash placing decode(lpad(to_hex($1), 2, '0'), 'hex') from 20),
                     info_dict, languages, release_name, release_group, description,
                     file_amount_per_type, uploaded_as_anonymous, file_list, mediainfo, trumpable,
                     staff_checked, container, size, duration, audio_codec, audio_bitrate,
                     audio_bitrate_sampling, audio_channels, video_codec, features,
                     subtitle_languages, video_resolution
                 FROM torrents WHERE id = 1",
            )
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        }
        sqlx::query("UPDATE torrents SET seeders = $2, leechers = $3 WHERE id = $1")
            .bind(id)
            .bind(seeders as i64)
            .bind(leechers as i64)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query(
        "INSERT INTO notifications_reseed_requests (torrent_id, user_id, requested_by_id)
         VALUES (1, 100, 197), (11, 100, 197), (12, 100, 197)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let db = ConnectionPool::with_pg_pool(pool.clone());
    let removed = db.remove_resolved_reseed_requests().await.unwrap();
    assert_eq!(removed, 1);

    let remaining: Vec<i32> = sqlx::query_scalar(
        "SELECT torrent_id FROM notifications_reseed_requests ORDER BY torrent_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(remaining, vec![11, 12]);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn marking_a_reseed_request_as_read_hides_it_without_deleting(pool: PgPool) {
    set_up_dead_torrent(&pool).await;
    let db = ConnectionPool::with_pg_pool(pool.clone());
    db.request_reseed(1, REQUESTER_ID, 72).await.unwrap();

    // the past seeder starts with one unread reseed notification
    let counts = db.find_notification_counts(PAST_SEEDER_ID).await.unwrap();
    assert_eq!(counts.reseed_requests, 1);

    db.mark_notifications_reseed_requests_as_read(PAST_SEEDER_ID, 1)
        .await
        .unwrap();

    // the row is kept, but no longer surfaced nor counted for the recipient
    assert_eq!(total_rows(&pool).await, 2);
    assert_eq!(reseed_rows(&pool, PAST_SEEDER_ID, true).await, 0);
    let notifications = db
        .find_all_notifications(PAST_SEEDER_ID, false)
        .await
        .unwrap();
    assert_eq!(notifications.reseed_requests.len(), 0);
    let counts = db.find_notification_counts(PAST_SEEDER_ID).await.unwrap();
    assert_eq!(counts.reseed_requests, 0);

    // when read notifications are included, the line is returned and flagged as read
    let with_read = db
        .find_all_notifications(PAST_SEEDER_ID, true)
        .await
        .unwrap();
    assert_eq!(with_read.reseed_requests.len(), 1);
    assert!(with_read.reseed_requests[0].read_status);

    // the other recipient is unaffected
    let other_counts = db.find_notification_counts(SNATCHER_ID).await.unwrap();
    assert_eq!(other_counts.reseed_requests, 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn auto_clear_removes_reseed_requests_of_soft_deleted_torrents(pool: PgPool) {
    set_up_dead_torrent(&pool).await;
    let db = ConnectionPool::with_pg_pool(pool.clone());
    db.request_reseed(1, REQUESTER_ID, 72).await.unwrap();
    assert_eq!(total_rows(&pool).await, 2);

    // the torrent is soft-deleted (ON DELETE CASCADE never fires), and it still has no seeders
    // so the healthy-again rule would not reach it
    sqlx::query("UPDATE torrents SET deleted_at = NOW() WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();

    let removed = db.remove_resolved_reseed_requests().await.unwrap();
    assert_eq!(removed, 2);
    assert_eq!(total_rows(&pool).await, 0);
}
