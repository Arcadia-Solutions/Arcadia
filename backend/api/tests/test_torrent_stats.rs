pub mod common;
pub mod mocks;

use std::sync::Arc;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool, models::torrent_stats::TorrentStatsResponse,
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{
    auth_header, call_and_read_body_json_with_status, create_test_app_and_login, TestUser,
};

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_no_grouping(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2025-01-01&to=2025-02-28&interval=month&group_by=none")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Should have 2 periods: Jan and Feb 2025
    assert_eq!(response.data.len(), 2);

    // 2 unique uploaders over the whole period (users 100 and 101)
    assert_eq!(response.unique_uploaders, 2);

    // site wide, whatever the period
    assert_eq!(response.total_seeding_size, 5000 + 3000);

    // January: 2 torrents (ids 10, 11), deleted one excluded
    let jan = &response.data[0];
    assert_eq!(jan.count, 2);
    assert_eq!(jan.total_size, 5000000000 + 3000000000);
    assert!(jan.attribute_value.is_none());

    // February: 1 torrent (id 12)
    let feb = &response.data[1];
    assert_eq!(feb.count, 1);
    assert_eq!(feb.total_size, 800000000);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_group_by_video_resolution(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2025-01-01&to=2025-01-31&interval=month&group_by=video_resolution")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // January has 2 non-deleted torrents with video_resolution set: 1080p and 720p
    assert_eq!(response.data.len(), 2);

    let resolutions: Vec<&str> = response
        .data
        .iter()
        .map(|d| d.attribute_value.as_deref().unwrap())
        .collect();
    assert!(resolutions.contains(&"1080p"));
    assert!(resolutions.contains(&"720p"));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_group_by_upload_method(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2025-01-01&to=2025-01-31&interval=month&group_by=upload_method")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // January has 2 non-deleted torrents: one uploaded manually, one through the api
    let upload_methods: Vec<&str> = response
        .data
        .iter()
        .map(|data_point| data_point.attribute_value.as_deref().unwrap())
        .collect();
    assert!(upload_methods.contains(&"manual"));
    assert!(upload_methods.contains(&"super-cli-tool"));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_group_by_content_type(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2025-01-01&to=2025-02-28&interval=month&group_by=content_type")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let content_types: Vec<&str> = response
        .data
        .iter()
        .map(|d| d.attribute_value.as_deref().unwrap())
        .collect();
    assert!(content_types.contains(&"movie"));
    assert!(content_types.contains(&"music"));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_excludes_deleted(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    // Query only January
    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2025-01-01&to=2025-01-31&interval=month&group_by=none")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.data.len(), 1);
    // Only 2 non-deleted torrents in January (not 3)
    assert_eq!(response.data[0].count, 2);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_fills_empty_periods(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    // Query Oct 2024 - Apr 2025: periods before the first torrent (Jan 2025) should be excluded,
    // but March (after first torrent, no data) should be filled with zeros
    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2024-10-01&to=2025-03-31&interval=month&group_by=none")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Only 3 periods: Jan, Feb, Mar (Oct-Dec excluded because no torrents existed yet)
    assert_eq!(response.data.len(), 3);

    // January: 2 torrents
    assert_eq!(response.data[0].count, 2);
    assert!(response.data[0].total_size > 0);

    // February: 1 torrent
    assert_eq!(response.data[1].count, 1);
    assert!(response.data[1].total_size > 0);

    // March: no data, should be filled with zeros
    assert_eq!(response.data[2].count, 0);
    assert_eq!(response.data[2].total_size, 0);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_title_groups_per_release_year(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2025-01-01&to=2025-02-28&interval=month&group_by=none")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // 2023: Movie C, 2025: Movie A + Album B, none: Movie D
    assert_eq!(response.title_groups_per_release_year.len(), 3);

    let year_2023 = &response.title_groups_per_release_year[0];
    assert_eq!(year_2023.year, Some(2023));
    assert_eq!(year_2023.count, 1);

    let year_2025 = &response.title_groups_per_release_year[1];
    assert_eq!(year_2025.year, Some(2025));
    assert_eq!(year_2025.count, 2);

    let no_release_date = &response.title_groups_per_release_year[2];
    assert_eq!(no_release_date.year, None);
    assert_eq!(no_release_date.count, 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_title_groups_per_release_year_respects_period(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    // All title groups in fixture were created on 2025-01-01, so querying 2024 returns none
    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2024-01-01&to=2024-12-31&interval=month&group_by=none")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert!(response.title_groups_per_release_year.is_empty());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_deletions(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2025-01-01&to=2025-02-28&interval=month&group_by=none")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.deletions.len(), 2);

    let jan = &response.deletions[0];
    assert_eq!(jan.count, 2);
    assert_eq!(jan.trumped, 1);
    assert_eq!(jan.duplicate, 1);
    assert_eq!(jan.other, 0);

    let feb = &response.deletions[1];
    assert_eq!(feb.count, 1);
    assert_eq!(feb.trumped, 0);
    assert_eq!(feb.duplicate, 0);
    assert_eq!(feb.other, 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_torrent_stats"),
    migrations = "../storage/migrations"
)]
async fn test_torrent_stats_snatches(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let req = test::TestRequest::get()
        .uri("/api/stats/torrents?from=2025-01-01&to=2025-02-28&interval=month&group_by=content_type")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: TorrentStatsResponse =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let point = |month: u32, content_type: &str| {
        response
            .data
            .iter()
            .find(|d| {
                d.period.format("%m").to_string() == format!("{month:02}")
                    && d.attribute_value.as_deref() == Some(content_type)
            })
            .unwrap()
    };

    // January: torrent 10 and the deleted torrent 13 (movies), the uncompleted grab is ignored
    assert_eq!(point(1, "movie").snatches, 2);
    assert_eq!(point(1, "movie").snatched_size, 5000000000 + 2000000000);
    assert_eq!(point(1, "music").snatches, 0);

    // February: torrent 12 (music) snatched twice, uploads are unaffected
    assert_eq!(point(2, "music").snatches, 2);
    assert_eq!(point(2, "music").snatched_size, 2 * 800000000);
    assert_eq!(point(2, "music").count, 1);
}
