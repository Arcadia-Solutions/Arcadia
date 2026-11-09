use crate::{common, mocks};

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::edition_group::{EditedEditionGroup, EditionGroup};
use common::auth_header;
use common::create_test_app_and_login;
use mocks::mock_redis::MockRedisPool;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"))]
async fn test_user_with_permission_can_edit_edition_group(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditEditionGroup).await;

    let req_body = EditedEditionGroup {
        id: 1,
        name: Some("Updated Edition Name".into()),
        release_date: Some(chrono::NaiveDate::from_ymd_opt(1962, 10, 5).unwrap()),
        release_date_only_year_known: false,
        description: Some("Updated description for the edition.".into()),
        distributor: Some("Updated Distributor".into()),
        covers: vec!["https://example.com/new-cover.jpg".into()],
        external_links: vec!["https://discogs.com/release/updated".into()],
        source: Some(arcadia_storage::models::edition_group::Source::Vinyl),
        additional_information: None,
    };

    let req = test::TestRequest::put()
        .uri("/api/edition-groups")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let response = common::call_and_read_body_json_with_status::<EditionGroup, _>(
        &service,
        req,
        StatusCode::OK,
    )
    .await;

    assert_eq!(response.name, req_body.name);
    assert_eq!(response.description, req_body.description);
    assert_eq!(response.distributor, req_body.distributor);
    assert_eq!(response.release_date, req_body.release_date);
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"))]
async fn test_user_without_permission_cannot_edit_edition_group(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req_body = EditedEditionGroup {
        id: 1,
        name: Some("Should Not Update".into()),
        release_date: Some(chrono::NaiveDate::from_ymd_opt(1962, 10, 5).unwrap()),
        release_date_only_year_known: false,
        description: Some("This should fail.".into()),
        distributor: Some("Test Distributor".into()),
        covers: vec![],
        external_links: vec![],
        source: None,
        additional_information: None,
    };

    let req = test::TestRequest::put()
        .uri("/api/edition-groups")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"))]
async fn test_user_with_permission_can_delete_edition_group(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteEditionGroup,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/edition-groups?edition_group_id=2")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify edition group was actually deleted
    let result = pool.find_edition_group(2).await;
    assert!(result.is_err());
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"))]
async fn test_user_without_permission_cannot_delete_edition_group(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .uri("/api/edition-groups?edition_group_id=2")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Verify edition group was NOT deleted
    let result = pool.find_edition_group(2).await;
    assert!(result.is_ok());
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"))]
async fn test_creator_can_delete_own_recent_edition_group(pool: PgPool) {
    let pool_arc = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Edition group 2 has created_by_id=1 and was created with NOW(), so it's recent.
    // Update it to be owned by user_basic (id=100) so the standard user is the creator.
    sqlx::query("UPDATE edition_groups SET created_by_id = 100 WHERE id = 2")
        .execute(std::borrow::Borrow::<sqlx::PgPool>::borrow(
            pool_arc.as_ref(),
        ))
        .await
        .unwrap();

    let (service, user) = create_test_app_and_login(
        pool_arc.clone(),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/edition-groups?edition_group_id=2")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify edition group was actually deleted
    let result = pool_arc.find_edition_group(2).await;
    assert!(result.is_err());
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"))]
async fn test_creator_cannot_delete_old_edition_group(pool: PgPool) {
    let pool_arc = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Edition group 1 has created_by_id=1 and created_at in 2025 (>24h ago).
    // Update it to be owned by user_basic (id=100).
    sqlx::query("UPDATE edition_groups SET created_by_id = 100 WHERE id = 1")
        .execute(std::borrow::Borrow::<sqlx::PgPool>::borrow(
            pool_arc.as_ref(),
        ))
        .await
        .unwrap();

    let (service, user) = create_test_app_and_login(
        pool_arc.clone(),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/edition-groups?edition_group_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Verify edition group was NOT deleted
    let result = pool_arc.find_edition_group(1).await;
    assert!(result.is_ok());
}

// --- Move torrent to edition group tests ---

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_movable_torrent"
))]
async fn test_user_with_permission_can_move_torrent(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::MoveTorrentToOtherEditionGroup,
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/torrents/move-to-edition-group")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "torrent_id": 3,
            "target_edition_group_id": 3
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let torrent = pool.find_torrent(3).await.unwrap();
    assert_eq!(torrent.edition_group_id, 3);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_movable_torrent"
))]
async fn test_user_with_permission_can_move_old_torrent(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::MoveTorrentToOtherEditionGroup,
    )
    .await;

    // Torrent 4 is older than 24h but permission holder can still move it
    let req = test::TestRequest::put()
        .uri("/api/torrents/move-to-edition-group")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "torrent_id": 4,
            "target_edition_group_id": 3
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let torrent = pool.find_torrent(4).await.unwrap();
    assert_eq!(torrent.edition_group_id, 3);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_movable_torrent"
))]
async fn test_creator_can_move_own_recent_torrent(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    // user_basic (id=100) is the creator of torrent 3 (created with NOW)
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::put()
        .uri("/api/torrents/move-to-edition-group")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "torrent_id": 3,
            "target_edition_group_id": 3
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let torrent = pool.find_torrent(3).await.unwrap();
    assert_eq!(torrent.edition_group_id, 3);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_movable_torrent"
))]
async fn test_creator_cannot_move_own_old_torrent(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    // user_basic (id=100) is the creator of torrent 4 (created 48h ago)
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::put()
        .uri("/api/torrents/move-to-edition-group")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "torrent_id": 4,
            "target_edition_group_id": 3
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Verify torrent was NOT moved
    let torrent = pool.find_torrent(4).await.unwrap();
    assert_eq!(torrent.edition_group_id, 1);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_movable_torrent"
))]
async fn test_non_creator_without_permission_cannot_move_torrent(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    // user_edit_art (id=101) is NOT the creator of torrent 3
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::EditArtist)
            .await;

    let req = test::TestRequest::put()
        .uri("/api/torrents/move-to-edition-group")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "torrent_id": 3,
            "target_edition_group_id": 3
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Verify torrent was NOT moved
    let torrent = pool.find_torrent(3).await.unwrap();
    assert_eq!(torrent.edition_group_id, 1);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_movable_torrent"
))]
async fn test_cannot_move_torrent_to_edition_group_in_different_title_group(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::MoveTorrentToOtherEditionGroup,
    )
    .await;

    // Edition group 2 belongs to title_group 2, torrent 3 belongs to edition_group 1 (title_group 1)
    let req = test::TestRequest::put()
        .uri("/api/torrents/move-to-edition-group")
        .insert_header(auth_header(&user.token))
        .set_json(json!({
            "torrent_id": 3,
            "target_edition_group_id": 2
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Verify torrent was NOT moved
    let torrent = pool.find_torrent(3).await.unwrap();
    assert_eq!(torrent.edition_group_id, 1);
}
