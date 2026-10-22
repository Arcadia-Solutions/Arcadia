pub mod common;
pub mod mocks;

use std::sync::Arc;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{connection_pool::ConnectionPool, models::torrent_report::TorrentReport};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{auth_header, create_test_app_and_login, TestUser};

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn test_create_and_delete_torrent_report(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Create torrent report as standard user
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .uri("/api/torrents/reports")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({
            "reported_torrent_id": 1,
            "description": "Test torrent report description"
        }))
        .to_request();

    let report: TorrentReport =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(report.reported_torrent_id, 1);
    assert_eq!(report.reported_by.id, 100);
    assert_eq!(report.description, "Test torrent report description");

    // Verify report exists in database
    assert!(
        pool.torrent_report_exists(report.id).await.unwrap(),
        "torrent report should exist in database"
    );

    // Delete report as user with delete_torrent_report permission
    let (service_delete, user_delete) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteTorrentReport,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!(
            "/api/torrents/reports?torrent_report_id={}",
            report.id
        ))
        .insert_header(auth_header(&user_delete.token))
        .to_request();

    let resp = test::call_service(&service_delete, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify report no longer exists in database
    assert!(
        !pool.torrent_report_exists(report.id).await.unwrap(),
        "torrent report should no longer exist in database"
    );
}
