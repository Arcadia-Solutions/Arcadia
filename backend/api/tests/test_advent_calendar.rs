pub mod common;
pub mod mocks;

use std::sync::Arc;

use actix_web::{
    http::StatusCode,
    test::{self, call_service},
};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{
        advent_calendar::{
            AdventCalendar, AdventCalendarDoorOpeningAttempt, AdventCalendarDoorOpeningResult,
            AdventCalendarDoorSettings, AdventCalendarDoorState, AdventCalendarLateOpeningPolicy,
            AdventCalendarRewardDistribution, PublishedAdventCalendar, UserCreatedAdventCalendar,
            UserCreatedAdventCalendarDoor,
        },
        bonus_points_log::{
            BonusPointsLogAction, BonusPointsLogOrderByColumn, SearchBonusPointsLogsQuery,
        },
        common::OrderByDirection,
    },
};
use chrono::{DateTime, Duration, Utc};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{
    auth_header, call_and_read_body_json_with_status, create_test_app_and_login, login_as, TestUser,
};

const STANDARD_USER_ID: i32 = 100;

fn door(name: &str) -> UserCreatedAdventCalendarDoor {
    UserCreatedAdventCalendarDoor {
        id: None,
        settings: AdventCalendarDoorSettings {
            name: name.to_string(),
            image_url: None,
            duration_minutes: 1440,
            opening_message: format!("Welcome to {name}"),
            locked_message: None,
            passphrase: None,
            staff_notes: String::new(),
            upload_min: 0,
            upload_max: 0,
            upload_distribution: AdventCalendarRewardDistribution::Linear,
            download_removed_min: 0,
            download_removed_max: 0,
            download_removed_distribution: AdventCalendarRewardDistribution::Linear,
            freeleech_tokens_min: 0,
            freeleech_tokens_max: 0,
            freeleech_tokens_distribution: AdventCalendarRewardDistribution::Linear,
            invitations_min: 0,
            invitations_max: 0,
            invitations_distribution: AdventCalendarRewardDistribution::Linear,
            bonus_points_min: 0,
            bonus_points_max: 0,
            bonus_points_distribution: AdventCalendarRewardDistribution::Linear,
        },
    }
}

fn calendar(
    starts_at: DateTime<Utc>,
    published: bool,
    doors: Vec<UserCreatedAdventCalendarDoor>,
) -> UserCreatedAdventCalendar {
    UserCreatedAdventCalendar {
        name: "Advent 2026".to_string(),
        starts_at,
        top_message: "Happy holidays".to_string(),
        button_text: "Advent calendar".to_string(),
        staff_notes: "staff only".to_string(),
        late_opening_policy: AdventCalendarLateOpeningPolicy::DuringDoorWindow,
        late_opening_grace_hours: 0,
        published,
        doors,
    }
}

fn create_request(token: &str, payload: &UserCreatedAdventCalendar) -> actix_http::Request {
    test::TestRequest::post()
        .uri("/api/advent-calendars")
        .insert_header(auth_header(token))
        .set_json(payload)
        .to_request()
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_create_and_get_advent_calendar(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;

    let mut second_door = door("Day 2");
    second_door.settings.bonus_points_min = 50;
    second_door.settings.bonus_points_max = 50;
    second_door.settings.passphrase = Some("Snow Flake".to_string());
    second_door.settings.duration_minutes = 60;
    let payload = calendar(Utc::now(), false, vec![door("Day 1"), second_door]);

    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;

    let request = test::TestRequest::get()
        .uri(&format!("/api/advent-calendars/{}", created.id))
        .insert_header(auth_header(&staff.token))
        .to_request();
    let fetched: AdventCalendar =
        call_and_read_body_json_with_status(&service, request, StatusCode::OK).await;

    assert_eq!(fetched.name, "Advent 2026");
    assert_eq!(fetched.staff_notes, "staff only");
    assert_eq!(fetched.doors.len(), 2);
    assert_eq!(fetched.doors[0].settings.name, "Day 1");
    assert_eq!(fetched.doors[0].position, 0);
    assert_eq!(fetched.doors[1].position, 1);
    assert_eq!(fetched.doors[1].settings.duration_minutes, 60);
    assert_eq!(
        fetched.doors[1].settings.passphrase.as_deref(),
        Some("Snow Flake")
    );
    assert_eq!(fetched.doors[1].settings.bonus_points_min, 50);
    assert_eq!(fetched.doors[1].settings.bonus_points_max, 50);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_update_calendar_reorders_and_removes_doors(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;

    let payload = calendar(
        Utc::now(),
        false,
        vec![door("Day 1"), door("Day 2"), door("Day 3")],
    );
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;
    let third_door_id = created.doors[2].id;

    let mut kept_door = door("Day 3 renamed");
    kept_door.id = Some(third_door_id);
    let update = calendar(Utc::now(), false, vec![kept_door, door("New door")]);
    let request = test::TestRequest::put()
        .uri(&format!("/api/advent-calendars/{}", created.id))
        .insert_header(auth_header(&staff.token))
        .set_json(&update)
        .to_request();
    let updated: AdventCalendar =
        call_and_read_body_json_with_status(&service, request, StatusCode::OK).await;

    assert_eq!(updated.doors.len(), 2);
    assert_eq!(updated.doors[0].id, third_door_id);
    assert_eq!(updated.doors[0].settings.name, "Day 3 renamed");
    assert_eq!(updated.doors[0].position, 0);
    assert_eq!(updated.doors[1].settings.name, "New door");
    assert_eq!(updated.doors[1].position, 1);

    // an id that is not a door of this calendar must not create a door with that id
    let mut unknown_door = door("Unknown");
    unknown_door.id = Some(third_door_id + 1000);
    let request = test::TestRequest::put()
        .uri(&format!("/api/advent-calendars/{}", created.id))
        .insert_header(auth_header(&staff.token))
        .set_json(calendar(Utc::now(), false, vec![unknown_door]))
        .to_request();
    let response = call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_publish_second_calendar_conflicts(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;

    let payload = calendar(Utc::now(), true, vec![door("Day 1")]);
    let first = call_service(&service, create_request(&staff.token, &payload)).await;
    assert_eq!(first.status(), StatusCode::OK);
    let second = call_service(&service, create_request(&staff.token, &payload)).await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_create_calendar_rejects_invalid_rewards(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;

    let mut invalid_door = door("Day 1");
    invalid_door.settings.upload_max = 10;
    invalid_door.settings.upload_distribution = AdventCalendarRewardDistribution::Logarithmic;
    let payload = calendar(Utc::now() + Duration::days(1), false, vec![invalid_door]);
    let response = call_service(&service, create_request(&staff.token, &payload)).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

fn open_request(token: &str, door_id: i32, passphrase: Option<&str>) -> actix_http::Request {
    test::TestRequest::post()
        .uri(&format!("/api/advent-calendars/doors/{door_id}/open"))
        .insert_header(auth_header(token))
        .set_json(AdventCalendarDoorOpeningAttempt {
            passphrase: passphrase.map(str::to_string),
        })
        .to_request()
}

fn published_request(token: &str) -> actix_http::Request {
    test::TestRequest::get()
        .uri("/api/advent-calendars/published")
        .insert_header(auth_header(token))
        .to_request()
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_open_door_applies_rewards(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let mut rewarding_door = door("Day 1");
    (
        rewarding_door.settings.upload_min,
        rewarding_door.settings.upload_max,
    ) = (1000, 1000);
    (
        rewarding_door.settings.bonus_points_min,
        rewarding_door.settings.bonus_points_max,
    ) = (50, 50);
    (
        rewarding_door.settings.invitations_min,
        rewarding_door.settings.invitations_max,
    ) = (2, 2);
    (
        rewarding_door.settings.freeleech_tokens_min,
        rewarding_door.settings.freeleech_tokens_max,
    ) = (1, 1);
    let payload = calendar(Utc::now() - Duration::hours(1), true, vec![rewarding_door]);
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;
    let before = pool.find_user_with_id(STANDARD_USER_ID).await.unwrap();

    let result: AdventCalendarDoorOpeningResult = call_and_read_body_json_with_status(
        &service,
        open_request(&user.token, created.doors[0].id, None),
        StatusCode::OK,
    )
    .await;

    assert_eq!(
        result,
        AdventCalendarDoorOpeningResult {
            opening_message: "Welcome to Day 1".to_string(),
            upload: 1000,
            download_removed: 0,
            freeleech_tokens: 1,
            invitations: 2,
            bonus_points: 50,
        }
    );
    let after = pool.find_user_with_id(STANDARD_USER_ID).await.unwrap();
    assert_eq!(after.uploaded, before.uploaded + 1000);
    assert_eq!(after.downloaded, before.downloaded);
    assert_eq!(after.bonus_points, before.bonus_points + 50);
    assert_eq!(after.invitations, before.invitations + 2);
    assert_eq!(after.freeleech_tokens, before.freeleech_tokens + 1);

    let logs = pool
        .search_bonus_points_logs(&SearchBonusPointsLogsQuery {
            page: 1,
            page_size: 10,
            order_by_column: BonusPointsLogOrderByColumn::CreatedAt,
            order_by_direction: OrderByDirection::Desc,
            from_date: Utc::now() - Duration::hours(1),
            to_date: Utc::now() + Duration::hours(1),
            actions: "advent_calendar_door_reward".to_string(),
            user_id: STANDARD_USER_ID,
        })
        .await
        .unwrap();
    assert_eq!(logs.results.len(), 1);
    assert!(matches!(
        logs.results[0].action,
        BonusPointsLogAction::AdventCalendarDoorReward
    ));
    assert_eq!(logs.results[0].amount, 50);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_open_door_twice_fails(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let mut rewarding_door = door("Day 1");
    (
        rewarding_door.settings.bonus_points_min,
        rewarding_door.settings.bonus_points_max,
    ) = (50, 50);
    let payload = calendar(Utc::now() - Duration::hours(1), true, vec![rewarding_door]);
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;
    let door_id = created.doors[0].id;

    let first = call_service(&service, open_request(&user.token, door_id, None)).await;
    assert_eq!(first.status(), StatusCode::OK);
    let after_first = pool.find_user_with_id(STANDARD_USER_ID).await.unwrap();
    let second = call_service(&service, open_request(&user.token, door_id, None)).await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
    let after_second = pool.find_user_with_id(STANDARD_USER_ID).await.unwrap();
    assert_eq!(after_second.bonus_points, after_first.bonus_points);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_download_removed_clamps_to_zero(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let mut rewarding_door = door("Day 1");
    (
        rewarding_door.settings.download_removed_min,
        rewarding_door.settings.download_removed_max,
    ) = (1_000_000_000_000_000, 1_000_000_000_000_000);
    let payload = calendar(Utc::now() - Duration::hours(1), true, vec![rewarding_door]);
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;

    let response = call_service(
        &service,
        open_request(&user.token, created.doors[0].id, None),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let after = pool.find_user_with_id(STANDARD_USER_ID).await.unwrap();
    assert_eq!(after.downloaded, 0);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_wrong_passphrase_rejected(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let mut protected_door = door("Day 1");
    protected_door.settings.passphrase = Some("Snow Flake".to_string());
    let payload = calendar(Utc::now() - Duration::hours(1), true, vec![protected_door]);
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;
    let door_id = created.doors[0].id;

    let missing = call_service(&service, open_request(&user.token, door_id, None)).await;
    assert_eq!(missing.status(), StatusCode::FORBIDDEN);
    let wrong = call_service(&service, open_request(&user.token, door_id, Some("nope"))).await;
    assert_eq!(wrong.status(), StatusCode::FORBIDDEN);
    let right = call_service(
        &service,
        open_request(&user.token, door_id, Some("  snow flake ")),
    )
    .await;
    assert_eq!(right.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_locked_and_missed_doors_not_openable(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let mut payload = calendar(
        Utc::now() - Duration::hours(25),
        true,
        vec![door("Day 1"), door("Day 2"), door("Day 3")],
    );
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;
    let door_ids: Vec<i32> = created.doors.iter().map(|door| door.id).collect();

    let missed = call_service(&service, open_request(&user.token, door_ids[0], None)).await;
    assert_eq!(missed.status(), StatusCode::FORBIDDEN);
    let current = call_service(&service, open_request(&user.token, door_ids[1], None)).await;
    assert_eq!(current.status(), StatusCode::OK);
    let locked = call_service(&service, open_request(&user.token, door_ids[2], None)).await;
    assert_eq!(locked.status(), StatusCode::FORBIDDEN);

    payload.late_opening_policy = AdventCalendarLateOpeningPolicy::GracePeriod;
    payload.late_opening_grace_hours = 2;
    for (door, id) in payload.doors.iter_mut().zip(&door_ids) {
        door.id = Some(*id);
    }
    let request = test::TestRequest::put()
        .uri(&format!("/api/advent-calendars/{}", created.id))
        .insert_header(auth_header(&staff.token))
        .set_json(&payload)
        .to_request();
    let _: AdventCalendar =
        call_and_read_body_json_with_status(&service, request, StatusCode::OK).await;
    let within_grace = call_service(&service, open_request(&user.token, door_ids[0], None)).await;
    assert_eq!(within_grace.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_published_calendar_hides_unopened_content(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let mut protected_door = door("Day 1");
    protected_door.settings.passphrase = Some("secret".to_string());
    protected_door.settings.locked_message = Some("Find the passphrase".to_string());
    let mut locked_door = door("Day 2");
    locked_door.settings.passphrase = Some("later".to_string());
    locked_door.settings.locked_message = Some("Come back tomorrow".to_string());
    let payload = calendar(
        Utc::now() - Duration::hours(1),
        true,
        vec![protected_door, locked_door],
    );
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;

    let published: PublishedAdventCalendar = call_and_read_body_json_with_status(
        &service,
        published_request(&user.token),
        StatusCode::OK,
    )
    .await;
    assert_eq!(published.top_message, "Happy holidays");
    assert_eq!(published.doors[0].state, AdventCalendarDoorState::Openable);
    assert!(published.doors[0].requires_passphrase);
    assert!(published.doors[0].opening.is_none());
    assert_eq!(
        published.doors[0].locked_message.as_deref(),
        Some("Find the passphrase")
    );
    assert_eq!(published.doors[1].state, AdventCalendarDoorState::Locked);
    assert!(published.doors[1].locked_message.is_none());

    let opened = call_service(
        &service,
        open_request(&user.token, created.doors[0].id, Some("secret")),
    )
    .await;
    assert_eq!(opened.status(), StatusCode::OK);

    let published: PublishedAdventCalendar = call_and_read_body_json_with_status(
        &service,
        published_request(&user.token),
        StatusCode::OK,
    )
    .await;
    assert_eq!(published.doors[0].state, AdventCalendarDoorState::Opened);
    assert_eq!(
        published.doors[0].opening.as_ref().unwrap().opening_message,
        "Welcome to Day 1"
    );
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_update_calendar_keeps_openings_of_kept_doors(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;
    let user = login_as(&service, TestUser::Standard).await;

    let mut payload = calendar(
        Utc::now() - Duration::hours(1),
        true,
        vec![door("Day 1"), door("Day 2")],
    );
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;
    let opened = call_service(
        &service,
        open_request(&user.token, created.doors[0].id, None),
    )
    .await;
    assert_eq!(opened.status(), StatusCode::OK);

    payload.doors.truncate(1);
    payload.doors[0].id = Some(created.doors[0].id);
    let request = test::TestRequest::put()
        .uri(&format!("/api/advent-calendars/{}", created.id))
        .insert_header(auth_header(&staff.token))
        .set_json(&payload)
        .to_request();
    let _: AdventCalendar =
        call_and_read_body_json_with_status(&service, request, StatusCode::OK).await;

    let published: PublishedAdventCalendar = call_and_read_body_json_with_status(
        &service,
        published_request(&user.token),
        StatusCode::OK,
    )
    .await;
    assert_eq!(published.doors.len(), 1);
    assert_eq!(published.doors[0].state, AdventCalendarDoorState::Opened);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_public_settings_button_text_only_while_running(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;

    let button_text = |service_response: serde_json::Value| {
        service_response["running_advent_calendar_button_text"].clone()
    };
    let public_settings_request = || {
        test::TestRequest::get()
            .uri("/api/arcadia-settings/public")
            .insert_header(auth_header(&staff.token))
            .to_request()
    };

    let settings: serde_json::Value =
        call_and_read_body_json_with_status(&service, public_settings_request(), StatusCode::OK)
            .await;
    assert_eq!(button_text(settings), serde_json::Value::Null);

    let mut payload = calendar(Utc::now() + Duration::hours(1), true, vec![door("Day 1")]);
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;
    let settings: serde_json::Value =
        call_and_read_body_json_with_status(&service, public_settings_request(), StatusCode::OK)
            .await;
    assert_eq!(button_text(settings), serde_json::Value::Null);

    payload.starts_at = Utc::now() - Duration::hours(1);
    payload.doors[0].id = Some(created.doors[0].id);
    let request = test::TestRequest::put()
        .uri(&format!("/api/advent-calendars/{}", created.id))
        .insert_header(auth_header(&staff.token))
        .set_json(&payload)
        .to_request();
    let _: AdventCalendar =
        call_and_read_body_json_with_status(&service, request, StatusCode::OK).await;
    let settings: serde_json::Value =
        call_and_read_body_json_with_status(&service, public_settings_request(), StatusCode::OK)
            .await;
    assert_eq!(button_text(settings), "Advent calendar");
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_reset_own_progress_allows_reopening(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageAdventCalendars,
    )
    .await;

    let mut rewarding_door = door("Day 1");
    (
        rewarding_door.settings.bonus_points_min,
        rewarding_door.settings.bonus_points_max,
    ) = (5, 50);
    rewarding_door.settings.bonus_points_distribution = AdventCalendarRewardDistribution::Gaussian;
    let payload = calendar(Utc::now() - Duration::hours(1), true, vec![rewarding_door]);
    let created: AdventCalendar = call_and_read_body_json_with_status(
        &service,
        create_request(&staff.token, &payload),
        StatusCode::OK,
    )
    .await;
    let door_id = created.doors[0].id;

    let result: AdventCalendarDoorOpeningResult = call_and_read_body_json_with_status(
        &service,
        open_request(&staff.token, door_id, None),
        StatusCode::OK,
    )
    .await;
    assert!((5..=50).contains(&result.bonus_points));

    let reset = test::TestRequest::delete()
        .uri(&format!("/api/advent-calendars/{}/progress", created.id))
        .insert_header(auth_header(&staff.token))
        .to_request();
    assert_eq!(call_service(&service, reset).await.status(), StatusCode::OK);

    let reopened = call_service(&service, open_request(&staff.token, door_id, None)).await;
    assert_eq!(reopened.status(), StatusCode::OK);
}
