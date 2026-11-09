use crate::{common, mocks};

use std::sync::Arc;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{
        common::{OrderByDirection, PaginatedResults},
        conversation::{ConversationSearchOrderByColumn, ConversationSearchQuery},
        torrent_request::{TorrentRequestUnfillHierarchy, TorrentRequestWithTitleGroupLite},
    },
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{TestUser, auth_header, read_body_json_data};

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_request"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_torrent_request",
    "with_test_torrent_request_vote"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_torrent_request",
    "with_test_torrent_request_vote"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_torrent_request",
    "with_test_torrent_request_vote"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_cd_torrent_request"
))]
async fn test_fill_torrent_request_not_matching_criteria(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // torrent_id=1 is a Vinyl release, but request 2 asks for a CD
    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrent-requests/fill")
        .set_json(serde_json::json!({
            "torrent_request_id": 2,
            "torrent_id": 1
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_request"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_request"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_requests_of_standard_user"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_requests_of_standard_user"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_requests_of_standard_user"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_requests_of_standard_user"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_requests_of_standard_user"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_deletable_title_group",
    "with_test_torrent_requests_of_standard_user"
))]
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

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_request"
))]
async fn test_torrent_request_vote_with_negative_bounty_is_refused(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrent-requests/vote")
        .set_json(serde_json::json!({
            "torrent_request_id": 1,
            "bounty_upload": 1,
            "bounty_bonus_points": -1000000
        }))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

fn unfill_request(token: &str, reason: &str) -> actix_http::Request {
    test::TestRequest::post()
        .insert_header(auth_header(token))
        .uri("/api/torrent-requests/unfill")
        .set_json(serde_json::json!({ "torrent_request_id": 5, "reason": reason }))
        .to_request()
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_filled_torrent_request"
))]
async fn test_author_can_unfill_torrent_request_and_bounty_is_taken_back(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, author) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;
    let filler_before = pool.find_user_with_id(101).await.unwrap();
    let uploader_before = pool.find_user_with_id(1).await.unwrap();

    let resp = test::call_service(&service, unfill_request(&author.token, "Wrong edition")).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let filler = pool.find_user_with_id(101).await.unwrap();
    assert_eq!(filler.uploaded, filler_before.uploaded - 500);
    assert_eq!(filler.bonus_points, filler_before.bonus_points - 25);
    assert_eq!(filler.requests_filled, 0);
    let uploader = pool.find_user_with_id(1).await.unwrap();
    assert_eq!(uploader.bonus_points, uploader_before.bonus_points - 25);
    assert!(
        pool.find_torrent_request(5)
            .await
            .unwrap()
            .filled_at
            .is_none()
    );

    // only users with the manage_unfilled_requests permission see the unfills
    let req = test::TestRequest::get()
        .insert_header(auth_header(&author.token))
        .uri("/api/torrent-requests?id=5")
        .to_request();
    let torrent_request: serde_json::Value = common::call_and_read_body_json(&service, req).await;
    assert_eq!(torrent_request["unfills"], serde_json::json!([]));

    // the request is not filled anymore
    let resp = test::call_service(&service, unfill_request(&author.token, "Wrong edition")).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_filled_torrent_request"
))]
async fn test_filler_without_permission_cannot_unfill_torrent_request(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, filler) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditArtist)
            .await;

    let resp = test::call_service(&service, unfill_request(&filler.token, "Wrong edition")).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_filled_torrent_request"
))]
async fn test_staff_can_unfill_torrent_request_and_see_the_unfills(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = common::create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageUnfilledRequests,
    )
    .await;

    let resp = test::call_service(&service, unfill_request(&staff.token, "  ")).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let resp = test::call_service(&service, unfill_request(&staff.token, "Wrong edition")).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::get()
        .insert_header(auth_header(&staff.token))
        .uri("/api/torrent-requests?id=5")
        .to_request();
    let torrent_request: serde_json::Value = common::call_and_read_body_json(&service, req).await;
    let unfills: Vec<TorrentRequestUnfillHierarchy> =
        serde_json::from_value(torrent_request["unfills"].clone()).unwrap();
    assert_eq!(unfills.len(), 1);
    assert_eq!(unfills[0].filled_by_user.id, 101);
    assert_eq!(unfills[0].filled_by_torrent_id, 1);
    assert_eq!(unfills[0].reason, "Wrong edition");
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_filled_torrent_request"
))]
async fn test_cannot_vote_on_filled_torrent_request(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrent-requests/vote")
        .set_json(serde_json::json!({
            "torrent_request_id": 5,
            "bounty_upload": 1,
            "bounty_bonus_points": 0
        }))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_test_filled_torrent_request"
))]
async fn test_filler_is_notified_when_their_fill_is_undone(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, author) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let resp = test::call_service(&service, unfill_request(&author.token, "Wrong edition")).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let search_query = ConversationSearchQuery {
        search_term: None,
        search_titles_only: false,
        page: 1,
        page_size: 50,
        user_id: None,
        order_by_column: ConversationSearchOrderByColumn::LastMessage,
        order_by_direction: OrderByDirection::Desc,
        all_conversations: false,
    };
    let conversations = pool
        .search_conversations(101, &search_query, false)
        .await
        .unwrap();
    let unfill_conversation = conversations
        .results
        .iter()
        .find(|conversation| {
            conversation.subject == "A torrent request you filled has been unfilled"
        })
        .expect("the filler did not receive the unfill message");
    assert_eq!(unfill_conversation.sender_id, 1);
}
