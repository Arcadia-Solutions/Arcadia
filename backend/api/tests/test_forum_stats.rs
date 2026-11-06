use crate::mocks;

use std::sync::Arc;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{connection_pool::ConnectionPool, models::forum_stats::ForumStatsResponse};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{
    TestUser, auth_header, call_and_read_body_json_with_status, create_test_app_and_login,
};

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_stats"),
    migrations = "../storage/migrations"
)]
async fn test_forum_stats_posts_no_grouping(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let request = test::TestRequest::get()
        .uri("/api/stats/forum?from=2025-01-01&to=2025-02-28&interval=month&group_by=none&metric=posts")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: ForumStatsResponse =
        call_and_read_body_json_with_status(&service, request, StatusCode::OK).await;

    // The initial migration seeds 1 thread and 1 post in 2024-xx; the fixture adds 3 threads and 5 posts
    // in Jan/Feb 2025. Restricting the period to 2025 isolates fixture data only.
    assert_eq!(response.total_threads_created, 3);
    assert_eq!(response.total_posts_created, 5);
    assert_eq!(response.unique_posters, 2);
    assert_eq!(response.unique_thread_creators, 2);
    // 5 posts / 3 threads
    assert!((response.average_posts_per_thread - 5.0 / 3.0).abs() < 1e-9);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_forum_stats"),
    migrations = "../storage/migrations"
)]
async fn test_forum_stats_threads_group_by_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ViewStatsDetails).await;

    let request = test::TestRequest::get()
        .uri("/api/stats/forum?from=2025-01-01&to=2025-02-28&interval=month&group_by=user&metric=threads")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: ForumStatsResponse =
        call_and_read_body_json_with_status(&service, request, StatusCode::OK).await;

    let usernames: Vec<&str> = response
        .data
        .iter()
        .filter_map(|d| d.attribute_value.as_deref())
        .collect();
    assert!(usernames.contains(&"user_basic"));
    assert!(usernames.contains(&"user_edit_art"));
}
