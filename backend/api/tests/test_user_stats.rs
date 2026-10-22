pub mod common;
pub mod mocks;

use std::collections::HashMap;
use std::sync::Arc;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{connection_pool::ConnectionPool, models::user_stats::UserStatsResponse};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{
    auth_header, call_and_read_body_json_with_status, create_test_app_and_login, TestUser,
};

async fn get_stats(pool: PgPool, query: &str) -> UserStatsResponse {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let request = test::TestRequest::get()
        .uri(&format!("/api/stats/users?{query}"))
        .insert_header(auth_header(&user.token))
        .to_request();

    call_and_read_body_json_with_status(&service, request, StatusCode::OK).await
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_stats"),
    migrations = "../storage/migrations"
)]
async fn test_user_stats_registrations(pool: PgPool) {
    // Only the fixture users are registered in 2025, every other user is created at NOW().
    let response = get_stats(pool, "from=2025-01-01&to=2025-02-28&interval=month").await;

    assert_eq!(response.new_users, 3);
    assert_eq!(response.data.len(), 2);

    // January: users 100 and 101
    assert_eq!(response.data[0].count, 2);
    // February: user 102
    assert_eq!(response.data[1].count, 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_stats"),
    migrations = "../storage/migrations"
)]
async fn test_user_stats_periods_start_at_the_first_registration(pool: PgPool) {
    // the first fixture user registered in january 2025, nothing to show before that
    let response = get_stats(pool, "from=2024-01-01&to=2025-02-28&interval=year").await;

    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].count, 3);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_stats"),
    migrations = "../storage/migrations"
)]
async fn test_user_stats_only_counts_the_selected_period(pool: PgPool) {
    // the 2 users registered in january, not the one of february
    let response = get_stats(pool, "from=2025-01-01&to=2025-01-31&interval=month").await;

    assert_eq!(response.new_users, 2);
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].count, 2);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_user_stats_torrent_clients"
    ),
    migrations = "../storage/migrations"
)]
async fn test_user_stats_torrent_clients_count_each_user_once(pool: PgPool) {
    let response = get_stats(pool, "from=2025-01-01&to=2025-02-28&interval=month").await;

    // User 100 seeds 2 torrents with qBittorrent, but is a single user, like the single
    // user 101 seeding with Deluge: both clients get the same weight.
    let clients: HashMap<&str, i64> = response
        .torrent_clients
        .iter()
        .map(|client| (client.client.as_str(), client.count))
        .collect();
    assert_eq!(clients.len(), 2);
    assert_eq!(clients["qBittorrent/4.5.0"], 1);
    assert_eq!(clients["Deluge/2.1.1"], 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_user_stats_torrent_clients"
    ),
    migrations = "../storage/migrations"
)]
async fn test_user_stats_torrent_clients_only_counts_the_selected_period(pool: PgPool) {
    // Only one of user 100's qBittorrent peers announced after the 16th of january, the
    // Deluge peer stopped announcing on the 12th.
    let response = get_stats(pool, "from=2025-01-16&to=2025-02-28&interval=month").await;

    let clients: HashMap<&str, i64> = response
        .torrent_clients
        .iter()
        .map(|client| (client.client.as_str(), client.count))
        .collect();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients["qBittorrent/4.5.0"], 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_user_stats",
        "with_test_user_stats_countries"
    ),
    migrations = "../storage/migrations"
)]
async fn test_user_stats_countries(pool: PgPool) {
    let response = get_stats(pool, "from=2025-01-01&to=2025-02-28&interval=month").await;

    // users with no located registration ip are left out
    let countries: Vec<(&str, i64)> = response
        .countries
        .iter()
        .map(|country| (country.country_code.as_str(), country.count))
        .collect();
    assert_eq!(countries, vec![("FR", 2), ("DE", 1)]);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_user_stats",
        "with_test_user_stats_countries"
    ),
    migrations = "../storage/migrations"
)]
async fn test_user_stats_countries_only_counts_the_selected_period(pool: PgPool) {
    // user 102 (DE) registered in february
    let response = get_stats(pool, "from=2025-01-01&to=2025-01-31&interval=month").await;

    let countries: Vec<(&str, i64)> = response
        .countries
        .iter()
        .map(|country| (country.country_code.as_str(), country.count))
        .collect();
    assert_eq!(countries, vec![("FR", 2)]);
}
