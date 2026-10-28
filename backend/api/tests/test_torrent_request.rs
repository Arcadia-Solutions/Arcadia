pub mod common;
pub mod mocks;

use std::sync::Arc;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{common::PaginatedResults, torrent_request::TorrentRequestWithTitleGroupLite},
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{auth_header, read_body_json_data, TestUser};

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_request"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_torrent_requests(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/search/torrent-requests?title_group_name=Love&order_by=created_at&order_by_direction=desc&include_filled=false")
        .to_request();

    let resp = test::call_service(&service, req).await;

    if resp.status() != StatusCode::OK {
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        eprintln!("Error response: {}", body_str);
        panic!("Expected 200 OK, got 500");
    }

    let results: PaginatedResults<TorrentRequestWithTitleGroupLite> =
        read_body_json_data(resp).await;

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].torrent_request.torrent_request.id, 1);
    assert_eq!(
        results.results[0].title_group.name,
        "Love Me Do / P.S. I Love You"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_torrent_request",
        "with_test_torrent_request_vote"
    ),
    migrations = "../storage/migrations"
)]
async fn test_fill_torrent_request_success(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrent-requests/fill")
        .set_json(serde_json::json!({
            "torrent_request_id": 1,
            "torrent_id": 1
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_torrent_request",
        "with_test_torrent_request_vote"
    ),
    migrations = "../storage/migrations"
)]
async fn test_fill_torrent_request_already_filled(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Fill the request first
    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrent-requests/fill")
        .set_json(serde_json::json!({
            "torrent_request_id": 1,
            "torrent_id": 1
        }))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_torrent_request",
        "with_test_torrent_request_vote"
    ),
    migrations = "../storage/migrations"
)]
async fn test_fill_torrent_request_wrong_title_group(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // torrent_id=2 is in title_group 2, but request is for title_group 1
    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrent-requests/fill")
        .set_json(serde_json::json!({
            "torrent_request_id": 1,
            "torrent_id": 2
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_request"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_delete_torrent_request_comment(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteTorrentRequestComment,
    )
    .await;

    let created_comment = pool
        .create_torrent_request_comment(
            1,
            100,
            "This request looks interesting",
            &tokio::sync::broadcast::channel(1).0,
        )
        .await
        .unwrap();

    let req = test::TestRequest::delete()
        .uri(&format!(
            "/api/torrent-requests/comment/{}",
            created_comment.id
        ))
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Deleting it a second time fails, proving it is gone
    let req = test::TestRequest::delete()
        .uri(&format!(
            "/api/torrent-requests/comment/{}",
            created_comment.id
        ))
        .insert_header(auth_header(&staff.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_request"
    ),
    migrations = "../storage/migrations"
)]
async fn test_author_without_permission_cannot_delete_torrent_request_comment(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    // The comment is created by the logged in user (user_basic)
    let created_comment = pool
        .create_torrent_request_comment(
            1,
            100,
            "This request looks interesting",
            &tokio::sync::broadcast::channel(1).0,
        )
        .await
        .unwrap();

    let req = test::TestRequest::delete()
        .uri(&format!(
            "/api/torrent-requests/comment/{}",
            created_comment.id
        ))
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_requests_of_standard_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_author_can_delete_torrent_request_without_other_voters(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrent-requests")
        .set_json(serde_json::json!({
            "id": 2,
            "refund_bounty": false,
            "message": null
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Deleting it a second time fails, proving it is gone
    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrent-requests")
        .set_json(serde_json::json!({
            "id": 2,
            "refund_bounty": false,
            "message": null
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_requests_of_standard_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_author_cannot_delete_torrent_request_with_other_voters(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrent-requests")
        .set_json(serde_json::json!({
            "id": 3,
            "refund_bounty": false,
            "message": null
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_requests_of_standard_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_delete_torrent_request_and_refund_the_bounty(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteTorrentRequest,
    )
    .await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&staff.token))
        .uri("/api/torrent-requests")
        .set_json(serde_json::json!({
            "id": 3,
            "refund_bounty": true,
            "message": "This request is not allowed"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let other_voter = pool.find_user_with_id(101).await.unwrap();
    assert_eq!(other_voter.uploaded, 3000);
    assert_eq!(other_voter.bonus_points, 200);
    // their comment on the request was deleted along with it
    assert_eq!(other_voter.request_comments, 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_requests_of_standard_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_delete_torrent_request_without_refunding_the_bounty(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteTorrentRequest,
    )
    .await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&staff.token))
        .uri("/api/torrent-requests")
        .set_json(serde_json::json!({
            "id": 3,
            "refund_bounty": false,
            "message": null
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let other_voter = pool.find_user_with_id(101).await.unwrap();
    assert_eq!(other_voter.uploaded, 0);
    assert_eq!(other_voter.bonus_points, 0);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_torrent_requests_of_standard_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_filled_torrent_request_cannot_be_deleted(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteTorrentRequest,
    )
    .await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&staff.token))
        .uri("/api/torrent-requests")
        .set_json(serde_json::json!({
            "id": 4,
            "refund_bounty": true,
            "message": null
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_deletable_title_group",
        "with_test_torrent_requests_of_standard_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_torrent_request_title_group_is_locked_once_voted_on_or_filled(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let edit_request = |torrent_request_id: i64| {
        test::TestRequest::put()
            .insert_header(auth_header(&user.token))
            .uri("/api/torrent-requests")
            .set_json(serde_json::json!({
                "id": torrent_request_id,
                "title_group_id": 3,
                "edition_name": null,
                "release_group": null,
                "description": "moved to another title group",
                "languages": [],
                "container": [],
                "source": [],
                "initial_vote": {
                    "torrent_request_id": torrent_request_id,
                    "bounty_upload": 0,
                    "bounty_bonus_points": 0
                },
                "audio_codec": [],
                "audio_channels": [],
                "audio_bitrate_sampling": [],
                "video_codec": [],
                "features": [],
                "subtitle_languages": [],
                "video_resolution": []
            }))
            .to_request()
    };

    // only voted on by its author
    let resp = test::call_service(&service, edit_request(2)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // voted on by another user
    let resp = test::call_service(&service, edit_request(3)).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // already filled
    let resp = test::call_service(&service, edit_request(4)).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
