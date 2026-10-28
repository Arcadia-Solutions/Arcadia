pub mod common;
pub mod mocks;

use std::sync::Arc;

use actix_http::Request;
use actix_web::{
    dev::{Service, ServiceResponse},
    http::StatusCode,
    test, Error,
};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{
        common::{OrderByDirection, PaginatedResults},
        conversation::{ConversationHierarchy, ConversationSearchResult},
        peer::PublicPeer,
        title_group::TitleGroupHierarchyLite,
        torrent::{TorrentSearch, TorrentSearchOrderByColumn},
        torrent_activity::{TorrentActivityAndTitleGroup, TorrentActivityAndUser},
        user::PublicProfile,
    },
};
use mocks::mock_redis::MockRedisPool;
use serde::{de::DeserializeOwned, Deserialize};
use sqlx::PgPool;

use crate::common::{auth_header, TestUser};

async fn build_test_torrent_upload_request(token: &str, release_group: &str) -> Request {
    use actix_multipart_rfc7578::client::multipart;

    let mut form = multipart::Form::default();

    form.add_text("release_name", "test release name");
    form.add_text("release_group", release_group);
    form.add_text("description", "This is a test description");
    form.add_text("uploaded_as_anonymous", "true");
    form.add_text("mediainfo", "test mediainfo");
    form.add_text("languages", "English");
    form.add_text("container", "MKV");
    form.add_text("edition_group_id", "1");
    form.add_text("duration", "3600");
    form.add_text("audio_codec", "flac");
    form.add_text("audio_bitrate", "1200");
    form.add_text("audio_channels", "5.1");
    form.add_text("audio_bitrate_sampling", "256");
    form.add_text("video_codec", "h264");
    form.add_text("features", "DV,HDR");
    form.add_text("subtitle_languages", "English,French");
    form.add_text("video_resolution", "1080p");
    form.add_text("extras", "");
    form.add_text("bonus_points_snatch_cost", "0");

    let torrent_data = bytes::Bytes::from_static(include_bytes!(
        "data/debian-12.10.0-i386-netinst.iso.torrent"
    ));

    form.add_reader_file(
        "torrent_file",
        std::io::Cursor::new(torrent_data),
        "torrent_file.torrent",
    );

    let content_type = form.content_type();

    let payload = actix_web::body::to_bytes(multipart::Body::from(form))
        .await
        .unwrap();

    test::TestRequest::post()
        .uri("/api/torrents")
        .insert_header(auth_header(token))
        .insert_header(("Content-Type", content_type))
        .set_payload(payload)
        .to_request()
}

async fn upload_test_torrent<S, T>(service: &S, token: &str, release_group: &str) -> T
where
    S: Service<Request, Response = ServiceResponse, Error = Error>,
    T: DeserializeOwned,
{
    let req = build_test_torrent_upload_request(token, release_group).await;
    common::call_and_read_body_json_with_status::<T, _>(service, req, StatusCode::CREATED).await
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
async fn test_valid_torrent(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/torrents?id=1")
        .to_request();

    let resp = test::call_service(&service, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    // A minimum set of definitions for assertions.
    #[derive(Debug, Deserialize)]
    struct Info {
        private: isize,
    }

    #[derive(Debug, Deserialize)]
    struct MetaInfo {
        info: Info,
        announce: String,
    }

    let metainfo = common::read_body_bencode::<MetaInfo, _>(resp)
        .await
        .expect("could not deserialize metainfo");

    assert_eq!(
        metainfo.info.private, 1,
        "expected downloaded torrent to be private"
    );

    let test_user_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    assert!(
        metainfo.announce.contains(test_user_passkey),
        "expected announce url to contain test_user passkey"
    );
    assert!(
        metainfo
            .announce
            .ends_with(&format!("/announce/{test_user_passkey}")),
        "expected announce url to end with /announce/{test_user_passkey}, got: {}",
        metainfo.announce
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"),
    migrations = "../storage/migrations"
)]
async fn test_upload_torrent(pool: PgPool) {
    let pg_pool = pool.clone();

    // Set bonus points given on upload BEFORE creating the app
    // (settings are loaded into memory at app creation time)
    let bonus_points_on_upload: i64 = 500;
    sqlx::query("UPDATE arcadia_settings SET bonus_points_given_on_upload = $1")
        .bind(bonus_points_on_upload)
        .execute(&pg_pool)
        .await
        .unwrap();

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Get user's initial bonus points
    let initial_bonus_points: i64 =
        sqlx::query_scalar("SELECT bonus_points FROM users WHERE id = 100")
            .fetch_one(&pg_pool)
            .await
            .unwrap();

    #[derive(Debug, Deserialize)]
    struct Torrent {
        edition_group_id: i64,
        created_by_id: i32,
    }

    let torrent: Torrent = upload_test_torrent(&service, &user.token, "TESTGRoUP").await;

    assert_eq!(torrent.edition_group_id, 1);
    assert_eq!(torrent.created_by_id, 100);

    // Verify bonus points were awarded
    let final_bonus_points: i64 =
        sqlx::query_scalar("SELECT bonus_points FROM users WHERE id = 100")
            .fetch_one(&pg_pool)
            .await
            .unwrap();

    assert_eq!(
        final_bonus_points,
        initial_bonus_points + bonus_points_on_upload,
        "expected user to receive {} bonus points for uploading",
        bonus_points_on_upload
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"),
    migrations = "../storage/migrations"
)]
async fn test_fill_torrent_request_uploader_only_within_first_hour(pool: PgPool) {
    #[derive(Debug, Deserialize)]
    struct CreatedTorrent {
        id: i32,
        created_by_id: i32,
    }

    #[derive(Debug, Deserialize)]
    struct CreatedTorrentRequest {
        id: i64,
    }

    #[derive(Debug, Deserialize)]
    struct ErrorResponse {
        error: String,
    }

    #[derive(Debug, Deserialize)]
    struct TorrentRequestAndAssociatedData {
        torrent_request: TorrentRequest,
    }

    #[derive(Debug, Deserialize)]
    struct TorrentRequest {
        filled_by_user_id: Option<i32>,
        filled_by_torrent_id: Option<i32>,
        filled_at: Option<String>,
    }

    let pg_pool = pool.clone();
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (service_uploader, uploader) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let (service_other_user, other_user) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditArtist,
    )
    .await;

    // Upload torrent as uploader.
    let uploaded_torrent: CreatedTorrent =
        upload_test_torrent(&service_uploader, &uploader.token, "TESTGROUP").await;

    assert_eq!(uploaded_torrent.created_by_id, 100);

    // Give the other user some upload so the bounty can be placed.
    sqlx::query("UPDATE users SET uploaded = 10000000 WHERE id = 101")
        .execute(&pg_pool)
        .await
        .unwrap();

    // Create a torrent request (in same title group) as the other user.
    let create_request_payload = serde_json::json!({
        "title_group_id": 1,
        "edition_name": null,
        "release_group": null,
        "description": "test request",
        "languages": [],
        "container": [],
        "source": [],
        "audio_codec": [],
        "audio_channels": [],
        "audio_bitrate_sampling": [],
        "video_codec": [],
        "features": [],
        "subtitle_languages": [],
        "video_resolution": [],
        "video_resolution_other_x": null,
        "video_resolution_other_y": null,
        "initial_vote": {
            "torrent_request_id": 0,
            "bounty_upload": 1048576,
            "bounty_bonus_points": 0
        }
    });

    let req = test::TestRequest::post()
        .uri("/api/torrent-requests")
        .insert_header(auth_header(&other_user.token))
        .set_json(create_request_payload.clone())
        .to_request();

    let created_request_uploader_fill: CreatedTorrentRequest =
        common::call_and_read_body_json_with_status(&service_other_user, req, StatusCode::CREATED)
            .await;

    // Uploader can fill within the first hour.
    let req = test::TestRequest::post()
        .uri("/api/torrent-requests/fill")
        .insert_header(auth_header(&uploader.token))
        .set_json(serde_json::json!({
            "torrent_request_id": created_request_uploader_fill.id,
            "torrent_id": uploaded_torrent.id
        }))
        .to_request();

    let resp = test::call_service(&service_uploader, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/torrent-requests?id={}",
            created_request_uploader_fill.id
        ))
        .insert_header(auth_header(&uploader.token))
        .to_request();

    let request_data: TorrentRequestAndAssociatedData =
        common::call_and_read_body_json_with_status(&service_uploader, req, StatusCode::OK).await;

    assert_eq!(
        request_data.torrent_request.filled_by_user_id,
        Some(100),
        "expected request to be filled by uploader"
    );
    assert_eq!(
        request_data.torrent_request.filled_by_torrent_id,
        Some(uploaded_torrent.id),
        "expected request to be filled by uploaded torrent"
    );
    assert!(
        request_data.torrent_request.filled_at.is_some(),
        "expected request to have filled_at timestamp"
    );

    // Create another torrent request to validate the non-uploader grace period behavior.
    let req = test::TestRequest::post()
        .uri("/api/torrent-requests")
        .insert_header(auth_header(&other_user.token))
        .set_json(create_request_payload)
        .to_request();

    let created_request = common::call_and_read_body_json_with_status::<CreatedTorrentRequest, _>(
        &service_other_user,
        req,
        StatusCode::CREATED,
    )
    .await;

    // Non-uploader cannot fill within 1 hour.
    let req = test::TestRequest::post()
        .uri("/api/torrent-requests/fill")
        .insert_header(auth_header(&other_user.token))
        .set_json(serde_json::json!({
            "torrent_request_id": created_request.id,
            "torrent_id": uploaded_torrent.id
        }))
        .to_request();

    let error = common::call_and_read_body_json_with_status::<ErrorResponse, _>(
        &service_other_user,
        req,
        StatusCode::CONFLICT,
    )
    .await;

    assert!(
        error
            .error
            .contains("only the torrent uploader can fill requests"),
        "expected grace period error message, got: {:?}",
        error
    );

    // Move torrent upload time into the past so the grace period has elapsed.
    sqlx::query(
        r#"
        UPDATE torrents
        SET created_at = NOW() - INTERVAL '2 hours'
        WHERE id = $1
        "#,
    )
    .bind(uploaded_torrent.id)
    .execute(&pg_pool)
    .await
    .unwrap();

    // Non-uploader can fill after the grace period.
    let req = test::TestRequest::post()
        .uri("/api/torrent-requests/fill")
        .insert_header(auth_header(&other_user.token))
        .set_json(serde_json::json!({
            "torrent_request_id": created_request.id,
            "torrent_id": uploaded_torrent.id
        }))
        .to_request();

    let resp = test::call_service(&service_other_user, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify request is filled.
    let req = test::TestRequest::get()
        .uri(&format!("/api/torrent-requests?id={}", created_request.id))
        .insert_header(auth_header(&other_user.token))
        .to_request();

    let request_data: TorrentRequestAndAssociatedData =
        common::call_and_read_body_json_with_status(&service_other_user, req, StatusCode::OK).await;

    assert_eq!(
        request_data.torrent_request.filled_by_user_id,
        Some(101),
        "expected request to be filled by other user"
    );
    assert_eq!(
        request_data.torrent_request.filled_by_torrent_id,
        Some(uploaded_torrent.id),
        "expected request to be filled by uploaded torrent"
    );
    assert!(
        request_data.torrent_request.filled_at.is_some(),
        "expected request to have filled_at timestamp"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_find_torrents_by_external_link(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let query = TorrentSearch {
        title_group_name: Some("https://en.wikipedia.org/wiki/RollerCoaster_Tycoon".to_string()),
        title_group_content_type: vec![],
        title_group_category: vec![],
        title_group_tags: None,
        title_group_include_empty_groups: true,
        edition_group_source: vec![],
        torrent_video_resolution: vec![],
        torrent_language: vec![],
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: None,
        torrent_snatched_by_id: None,
        artist_id: None,
        collage_id: None,
        page: 1,
        page_size: 50,
        order_by_column: TorrentSearchOrderByColumn::TorrentCreatedAt,
        order_by_direction: OrderByDirection::Desc,
        series_id: None,
        user_id_bookmarks: None,
    };

    let query = serde_qs::to_string(&query).unwrap();
    let uri = format!("/api/search/torrents/lite?{}", query);

    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let groups = results.results;
    assert!(
        groups
            .iter()
            .any(|g| g.id == 2 && g.name == "RollerCoaster Tycoon"),
        "expected results to include title_group id=2 (RollerCoaster Tycoon) when searching by external link"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_find_torrents_by_external_link_with_trailing_slash(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let query = TorrentSearch {
        title_group_name: Some("https://en.wikipedia.org/wiki/RollerCoaster_Tycoon/".to_string()),
        title_group_content_type: vec![],
        title_group_category: vec![],
        title_group_tags: None,
        title_group_include_empty_groups: true,
        edition_group_source: vec![],
        torrent_video_resolution: vec![],
        torrent_language: vec![],
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: None,
        torrent_snatched_by_id: None,
        artist_id: None,
        collage_id: None,
        page: 1,
        page_size: 50,
        order_by_column: TorrentSearchOrderByColumn::TorrentCreatedAt,
        order_by_direction: OrderByDirection::Desc,
        series_id: None,
        user_id_bookmarks: None,
    };

    let query = serde_qs::to_string(&query).unwrap();
    let uri = format!("/api/search/torrents/lite?{}", query);

    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let groups = results.results;
    assert!(
        groups
            .iter()
            .any(|g| g.id == 2 && g.name == "RollerCoaster Tycoon"),
        "expected trailing slash in external link to match stored link without trailing slash"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_find_torrents_by_name(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let query = TorrentSearch {
        title_group_name: Some("Love Me Do".to_string()),
        title_group_content_type: vec![],
        title_group_category: vec![],
        title_group_tags: None,
        title_group_include_empty_groups: true,
        edition_group_source: vec![],
        torrent_video_resolution: vec![],
        torrent_language: vec![],
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: None,
        torrent_snatched_by_id: None,
        artist_id: None,
        collage_id: None,
        page: 1,
        page_size: 50,
        order_by_column: TorrentSearchOrderByColumn::TorrentCreatedAt,
        order_by_direction: OrderByDirection::Desc,
        series_id: None,
        user_id_bookmarks: None,
    };

    let query = serde_qs::to_string(&query).unwrap();
    let uri = format!("/api/search/torrents/lite?{}", query);

    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let groups = results.results;
    assert!(
        groups
            .iter()
            .any(|g| g.id == 1 && g.name == "Love Me Do / P.S. I Love You"),
        "expected results to include title_group id=1 (Love Me Do / P.S. I Love You) when searching by name"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_find_torrents_no_link_or_name_provided(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let query = TorrentSearch {
        title_group_name: Some("".to_string()),
        title_group_content_type: vec![],
        title_group_category: vec![],
        title_group_tags: None,
        title_group_include_empty_groups: true,
        edition_group_source: vec![],
        torrent_video_resolution: vec![],
        torrent_language: vec![],
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: None,
        torrent_snatched_by_id: None,
        artist_id: None,
        collage_id: None,
        page: 1,
        page_size: 50,
        order_by_column: TorrentSearchOrderByColumn::TorrentCreatedAt,
        order_by_direction: OrderByDirection::Desc,
        series_id: None,
        user_id_bookmarks: None,
    };

    let query = serde_qs::to_string(&query).unwrap();
    let uri = format!("/api/search/torrents/lite?{}", query);

    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let groups = results.results;
    let ids: Vec<i32> = groups.iter().map(|g| g.id).collect();

    assert!(
        ids.contains(&1) && ids.contains(&2),
        "expected unfiltered results to include both title_group id=1 and id=2"
    );
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
async fn test_set_torrent_staff_checked_with_permission(pool: PgPool) {
    use serde_json::json;

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::SetTorrentStaffChecked,
    )
    .await;

    let payload = json!({
        "torrent_id": 1,
        "staff_checked": true
    });

    let req = test::TestRequest::put()
        .uri("/api/torrents/staff-checked")
        .insert_header(auth_header(&user.token))
        .set_json(&payload)
        .to_request();

    let response: serde_json::Value =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response["result"], "success");

    // Verify the torrent's staff_checked field was updated
    let torrent = pool.find_torrent(1).await.unwrap();
    assert!(torrent.staff_checked);
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
async fn test_set_torrent_staff_checked_without_permission(pool: PgPool) {
    use serde_json::json;

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let payload = json!({
        "torrent_id": 1,
        "staff_checked": true
    });

    let req = test::TestRequest::put()
        .uri("/api/torrents/staff-checked")
        .insert_header(auth_header(&user.token))
        .set_json(&payload)
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::FORBIDDEN,
    )
    .await;
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_peers"
    ),
    migrations = "../storage/migrations"
)]
async fn test_get_torrent_peers_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = common::create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ViewTorrentPeersAndActivities,
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/torrents/peers?torrent_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let peers: Vec<PublicPeer> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Should return 3 active peers
    assert_eq!(peers.len(), 3, "expected 3 active peers");

    // Own peer (user_id 139) should have IP and port visible
    let own_peer = peers.iter().find(|p| p.user.id == 139).unwrap();
    assert!(own_peer.ip.is_some(), "own peer should have IP visible");
    assert!(own_peer.port.is_some(), "own peer should have port visible");

    // Other peers should have IP and port hidden
    let other_peer = peers.iter().find(|p| p.user.id == 100).unwrap();
    assert!(other_peer.ip.is_none(), "other peer should have IP hidden");
    assert!(
        other_peer.port.is_none(),
        "other peer should have port hidden"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_peers"
    ),
    migrations = "../storage/migrations"
)]
async fn test_get_torrent_peers_without_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/torrents/peers?torrent_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::FORBIDDEN,
    )
    .await;
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_seeding_size",
        "with_test_non_seeding_torrent_activity"
    ),
    migrations = "../storage/migrations"
)]
async fn test_get_torrent_activities_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = common::create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ViewTorrentPeersAndActivities,
    )
    .await;

    // same grabbed_at for every activity, so they are ordered by most recent activity first
    let req = test::TestRequest::get()
        .uri("/api/torrents/activities?torrent_id=1&page=1&page_size=2&order_by_column=grabbed_at&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let first_page: PaginatedResults<TorrentActivityAndUser> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(first_page.total_items, 3);
    let users_and_seeding: Vec<(i32, bool, bool)> = first_page
        .results
        .iter()
        .map(|activity| {
            (
                activity.user.id,
                activity.seeder,
                activity.completed_at.is_some(),
            )
        })
        .collect();
    assert_eq!(
        users_and_seeding,
        vec![(102, false, false), (101, true, true)]
    );

    let req = test::TestRequest::get()
        .uri("/api/torrents/activities?torrent_id=1&page=2&page_size=2&order_by_column=grabbed_at&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let second_page: PaginatedResults<TorrentActivityAndUser> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(second_page.results.len(), 1);
    assert_eq!(second_page.results[0].user.id, 100);
    assert!(second_page.results[0].seeder);

    let req = test::TestRequest::get()
        .uri("/api/torrents/activities?torrent_id=1&page=1&page_size=3&order_by_column=seeder&order_by_direction=asc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let sorted_by_seeder: PaginatedResults<TorrentActivityAndUser> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(sorted_by_seeder.results[0].user.id, 102);
    assert!(!sorted_by_seeder.results[0].seeder);
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
async fn test_edit_torrent_up_down_factors_with_permission(pool: PgPool) {
    use serde_json::json;

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditTorrentUpDownFactors,
    )
    .await;

    let payload = json!({
        "torrent_id": 1,
        "upload_factor": 200,
        "download_factor": 50
    });

    let req = test::TestRequest::put()
        .uri("/api/torrents/up-down-factors")
        .insert_header(auth_header(&user.token))
        .set_json(&payload)
        .to_request();

    let _response: serde_json::Value =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let torrent = pool.find_torrent(1).await.unwrap();
    assert_eq!(torrent.upload_factor, 200);
    assert_eq!(torrent.download_factor, 50);
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
async fn test_edit_torrent_up_down_factors_without_permission(pool: PgPool) {
    use serde_json::json;

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let payload = json!({
        "torrent_id": 1,
        "upload_factor": 200,
        "download_factor": 50
    });

    let req = test::TestRequest::put()
        .uri("/api/torrents/up-down-factors")
        .insert_header(auth_header(&user.token))
        .set_json(&payload)
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::FORBIDDEN,
    )
    .await;
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"),
    migrations = "../storage/migrations"
)]
async fn test_upload_torrent_blocked_by_release_date_cutoff(pool: PgPool) {
    #[derive(Debug, Deserialize)]
    struct ErrorResponse {
        error: String,
    }

    let pg_pool = pool.clone();

    // Set torrent_max_release_date_allowed to 1960-01-01
    // The test fixtures have content with release dates in 1962 and 1999
    sqlx::query("UPDATE arcadia_settings SET torrent_max_release_date_allowed = '1960-01-01'")
        .execute(&pg_pool)
        .await
        .unwrap();

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    use actix_multipart_rfc7578::client::multipart;

    let mut form = multipart::Form::default();
    form.add_text("release_name", "test release name");
    form.add_text("release_group", "TESTGROUP");
    form.add_text("description", "This is a test description");
    form.add_text("uploaded_as_anonymous", "true");
    form.add_text("mediainfo", "test mediainfo");
    form.add_text("languages", "English");
    form.add_text("container", "MKV");
    form.add_text("edition_group_id", "1"); // Edition group with 1962 release date
    form.add_text("duration", "3600");
    form.add_text("audio_codec", "flac");
    form.add_text("audio_bitrate", "1200");
    form.add_text("audio_channels", "5.1");
    form.add_text("audio_bitrate_sampling", "256");
    form.add_text("video_codec", "h264");
    form.add_text("features", "");
    form.add_text("subtitle_languages", "");
    form.add_text("video_resolution", "1080p");
    form.add_text("extras", "");
    form.add_text("bonus_points_snatch_cost", "0");

    let torrent_data = bytes::Bytes::from_static(include_bytes!(
        "data/debian-12.10.0-i386-netinst.iso.torrent"
    ));

    form.add_reader_file(
        "torrent_file",
        std::io::Cursor::new(torrent_data),
        "torrent_file.torrent",
    );

    let content_type = form.content_type();

    let payload = actix_web::body::to_bytes(multipart::Body::from(form))
        .await
        .unwrap();

    let req = test::TestRequest::post()
        .uri("/api/torrents")
        .insert_header(auth_header(&user.token))
        .insert_header(("Content-Type", content_type))
        .set_payload(payload)
        .to_request();

    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;

    assert!(
        error
            .error
            .contains("content released after 1960-01-01 is not allowed"),
        "expected release date cutoff error, got: {:?}",
        error
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"),
    migrations = "../storage/migrations"
)]
async fn test_upload_torrent_bonus_points_cost_above_max(pool: PgPool) {
    #[derive(Debug, Deserialize)]
    struct ErrorResponse {
        error: String,
    }

    let pg_pool = pool.clone();

    sqlx::query("UPDATE arcadia_settings SET allow_uploader_set_torrent_bonus_points_cost = true, torrent_bonus_points_cost_min = 10, torrent_bonus_points_cost_max = 100")
        .execute(&pg_pool)
        .await
        .unwrap();

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    use actix_multipart_rfc7578::client::multipart;

    let mut form = multipart::Form::default();
    form.add_text("release_name", "test release name");
    form.add_text("release_group", "TESTGROUP");
    form.add_text("description", "This is a test description");
    form.add_text("uploaded_as_anonymous", "true");
    form.add_text("mediainfo", "test mediainfo");
    form.add_text("languages", "English");
    form.add_text("container", "MKV");
    form.add_text("edition_group_id", "1");
    form.add_text("duration", "3600");
    form.add_text("audio_codec", "flac");
    form.add_text("audio_bitrate", "1200");
    form.add_text("audio_channels", "5.1");
    form.add_text("audio_bitrate_sampling", "256");
    form.add_text("video_codec", "h264");
    form.add_text("features", "");
    form.add_text("subtitle_languages", "");
    form.add_text("video_resolution", "1080p");
    form.add_text("extras", "");
    form.add_text("bonus_points_snatch_cost", "150");

    let torrent_data = bytes::Bytes::from_static(include_bytes!(
        "data/debian-12.10.0-i386-netinst.iso.torrent"
    ));

    form.add_reader_file(
        "torrent_file",
        std::io::Cursor::new(torrent_data),
        "torrent_file.torrent",
    );

    let content_type = form.content_type();

    let payload = actix_web::body::to_bytes(multipart::Body::from(form))
        .await
        .unwrap();

    let req = test::TestRequest::post()
        .uri("/api/torrents")
        .insert_header(auth_header(&user.token))
        .insert_header(("Content-Type", content_type))
        .set_payload(payload)
        .to_request();

    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;

    assert!(
        error
            .error
            .contains("bonus_points_snatch_cost must be at most 100"),
        "expected bonus points cost max error, got: {:?}",
        error
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group", "with_test_edition_group"),
    migrations = "../storage/migrations"
)]
async fn test_upload_torrent_bonus_points_cost_below_min(pool: PgPool) {
    #[derive(Debug, Deserialize)]
    struct ErrorResponse {
        error: String,
    }

    let pg_pool = pool.clone();

    sqlx::query("UPDATE arcadia_settings SET allow_uploader_set_torrent_bonus_points_cost = true, torrent_bonus_points_cost_min = 10, torrent_bonus_points_cost_max = 100")
        .execute(&pg_pool)
        .await
        .unwrap();

    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    use actix_multipart_rfc7578::client::multipart;

    let mut form = multipart::Form::default();
    form.add_text("release_name", "test release name");
    form.add_text("release_group", "TESTGROUP2");
    form.add_text("description", "This is a test description");
    form.add_text("uploaded_as_anonymous", "true");
    form.add_text("mediainfo", "test mediainfo");
    form.add_text("languages", "English");
    form.add_text("container", "MKV");
    form.add_text("edition_group_id", "1");
    form.add_text("duration", "3600");
    form.add_text("audio_codec", "flac");
    form.add_text("audio_bitrate", "1200");
    form.add_text("audio_channels", "5.1");
    form.add_text("audio_bitrate_sampling", "256");
    form.add_text("video_codec", "h264");
    form.add_text("features", "");
    form.add_text("subtitle_languages", "");
    form.add_text("video_resolution", "1080p");
    form.add_text("extras", "");
    form.add_text("bonus_points_snatch_cost", "5");

    let torrent_data = bytes::Bytes::from_static(include_bytes!(
        "data/debian-12.10.0-i386-netinst.iso.torrent"
    ));

    form.add_reader_file(
        "torrent_file",
        std::io::Cursor::new(torrent_data),
        "torrent_file.torrent",
    );

    let content_type = form.content_type();

    let payload = actix_web::body::to_bytes(multipart::Body::from(form))
        .await
        .unwrap();

    let req = test::TestRequest::post()
        .uri("/api/torrents")
        .insert_header(auth_header(&user.token))
        .insert_header(("Content-Type", content_type))
        .set_payload(payload)
        .to_request();

    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;

    assert!(
        error
            .error
            .contains("bonus_points_snatch_cost must be at least 10"),
        "expected bonus points cost min error, got: {:?}",
        error
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_title_group_bookmark",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_torrents_by_own_bookmarks(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let query = TorrentSearch {
        title_group_name: None,
        title_group_content_type: vec![],
        title_group_category: vec![],
        title_group_tags: None,
        title_group_include_empty_groups: false,
        edition_group_source: vec![],
        torrent_video_resolution: vec![],
        torrent_language: vec![],
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: None,
        torrent_snatched_by_id: None,
        artist_id: None,
        collage_id: None,
        page: 1,
        page_size: 50,
        order_by_column: TorrentSearchOrderByColumn::TorrentCreatedAt,
        order_by_direction: OrderByDirection::Desc,
        series_id: None,
        user_id_bookmarks: Some(100),
    };

    let query = serde_qs::to_string(&query).unwrap();
    let uri = format!("/api/search/torrents/lite?{}", query);

    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(
        results.results.len(),
        1,
        "expected exactly one bookmarked title group"
    );
    assert_eq!(
        results.results[0].id, 1,
        "expected bookmarked title group to be id=1"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_create_bookmark_then_search(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // search by bookmarks before creating any — should return no results
    let query = TorrentSearch {
        title_group_name: None,
        title_group_content_type: vec![],
        title_group_category: vec![],
        title_group_tags: None,
        title_group_include_empty_groups: false,
        edition_group_source: vec![],
        torrent_video_resolution: vec![],
        torrent_language: vec![],
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: None,
        torrent_snatched_by_id: None,
        artist_id: None,
        collage_id: None,
        page: 1,
        page_size: 50,
        order_by_column: TorrentSearchOrderByColumn::TorrentCreatedAt,
        order_by_direction: OrderByDirection::Desc,
        series_id: None,
        user_id_bookmarks: Some(100),
    };

    let query_string = serde_qs::to_string(&query).unwrap();
    let uri = format!("/api/search/torrents/lite?{}", query_string);

    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert!(
        results.results.is_empty(),
        "expected no bookmarked title groups before creating a bookmark"
    );

    // create a bookmark on title group 1
    let req = test::TestRequest::post()
        .uri("/api/title-group-bookmarks")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({
            "title_group_id": 1,
            "description": "test bookmark"
        }))
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::CREATED,
    )
    .await;

    // search by bookmarks again — should now return the bookmarked title group
    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(
        results.results.len(),
        1,
        "expected exactly one bookmarked title group after creating a bookmark"
    );
    assert_eq!(
        results.results[0].id, 1,
        "expected bookmarked title group to be id=1"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_title_group_bookmark",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_torrents_by_other_user_bookmarks_forbidden(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let query = TorrentSearch {
        title_group_name: None,
        title_group_content_type: vec![],
        title_group_category: vec![],
        title_group_tags: None,
        title_group_include_empty_groups: false,
        edition_group_source: vec![],
        torrent_video_resolution: vec![],
        torrent_language: vec![],
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: None,
        torrent_snatched_by_id: None,
        artist_id: None,
        collage_id: None,
        page: 1,
        page_size: 50,
        order_by_column: TorrentSearchOrderByColumn::TorrentCreatedAt,
        order_by_direction: OrderByDirection::Desc,
        series_id: None,
        user_id_bookmarks: Some(999),
    };

    let query = serde_qs::to_string(&query).unwrap();
    let uri = format!("/api/search/torrents/lite?{}", query);

    let req = test::TestRequest::get()
        .uri(&uri)
        .insert_header(auth_header(&user.token))
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::FORBIDDEN,
    )
    .await;
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_torrent_bonus_points_snatch_costs",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_torrents_ordered_by_bonus_points_snatch_cost(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let query = TorrentSearch {
        title_group_name: None,
        title_group_content_type: vec![],
        title_group_category: vec![],
        title_group_tags: None,
        title_group_include_empty_groups: false,
        edition_group_source: vec![],
        torrent_video_resolution: vec![],
        torrent_language: vec![],
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: None,
        torrent_snatched_by_id: None,
        artist_id: None,
        collage_id: None,
        page: 1,
        page_size: 50,
        order_by_column: TorrentSearchOrderByColumn::TorrentBonusPointsSnatchCost,
        order_by_direction: OrderByDirection::Asc,
        series_id: None,
        user_id_bookmarks: None,
    };

    let search = |query: &TorrentSearch| {
        let uri = format!(
            "/api/search/torrents/lite?{}",
            serde_qs::to_string(query).unwrap()
        );
        test::TestRequest::get()
            .uri(&uri)
            .insert_header(auth_header(&user.token))
            .to_request()
    };

    // torrent 1 (title group 1) costs 500, torrent 2 (title group 2) costs 100
    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, search(&query), StatusCode::OK).await;

    let ascending_ids: Vec<i32> = results.results.iter().map(|group| group.id).collect();
    assert_eq!(
        ascending_ids,
        vec![2, 1],
        "expected the cheapest title group first when ordering ascending"
    );

    let descending_query = TorrentSearch {
        order_by_direction: OrderByDirection::Desc,
        ..query
    };

    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(
            &service,
            search(&descending_query),
            StatusCode::OK,
        )
        .await;

    let descending_ids: Vec<i32> = results.results.iter().map(|group| group.id).collect();
    assert_eq!(
        descending_ids,
        vec![1, 2],
        "expected the most expensive title group first when ordering descending"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_trumpable_torrent",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_torrents_by_trumpable(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // torrent 1 (title group 1) has an empty trumpable value, which means "not trumpable",
    // and torrent 2 (title group 2) has a real trumpable reason
    let query = TorrentSearch {
        title_group_name: None,
        title_group_content_type: vec![],
        title_group_category: vec![],
        title_group_tags: None,
        title_group_include_empty_groups: false,
        edition_group_source: vec![],
        torrent_video_resolution: vec![],
        torrent_language: vec![],
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: None,
        torrent_snatched_by_id: None,
        artist_id: None,
        collage_id: None,
        page: 1,
        page_size: 50,
        order_by_column: TorrentSearchOrderByColumn::TorrentCreatedAt,
        order_by_direction: OrderByDirection::Desc,
        series_id: None,
        user_id_bookmarks: None,
    };

    let search = |query: &TorrentSearch| {
        let uri = format!(
            "/api/search/torrents/lite?{}",
            serde_qs::to_string(query).unwrap()
        );
        test::TestRequest::get()
            .uri(&uri)
            .insert_header(auth_header(&user.token))
            .to_request()
    };

    let trumpable_query = TorrentSearch {
        torrent_trumpable: Some(true),
        ..query.clone()
    };
    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(
            &service,
            search(&trumpable_query),
            StatusCode::OK,
        )
        .await;
    let trumpable_ids: Vec<i32> = results.results.iter().map(|group| group.id).collect();
    assert_eq!(
        trumpable_ids,
        vec![2],
        "selecting 'yes' should only return title groups holding a torrent with a trumpable reason"
    );

    let not_trumpable_query = TorrentSearch {
        torrent_trumpable: Some(false),
        ..query
    };
    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(
            &service,
            search(&not_trumpable_query),
            StatusCode::OK,
        )
        .await;
    let not_trumpable_ids: Vec<i32> = results.results.iter().map(|group| group.id).collect();
    assert_eq!(
        not_trumpable_ids,
        vec![1],
        "selecting 'no' should not return torrents whose trumpable value is an empty string"
    );
}

/// Release name of torrent 1, from the `with_test_torrent` fixture.
const TORRENT_RELEASE_NAME: &str =
    "The Beatles - Love Me Do - P.S. I Love You (Parlophone Single) [24-96]";

/// Torrent 1, from the `with_test_torrent` fixture, uploaded by user id 1.
const TORRENT_ID: i32 = 1;

/// Torrent 900, from the `with_test_torrents_of_basic_user` fixture, uploaded by the basic user.
const TORRENT_OF_BASIC_USER_ID: i32 = 900;
const TORRENT_OF_BASIC_USER_RELEASE_NAME: &str = "Anonymously uploaded torrent";

/// Torrent 903, from the `with_test_torrent_of_edit_trump_user` fixture, uploaded by the user
/// holding the `edit_torrent_trumpable` permission.
const TORRENT_OF_EDIT_TRUMP_USER_ID: i32 = 903;
const TORRENT_OF_EDIT_TRUMP_USER_RELEASE_NAME: &str =
    "Torrent of a user allowed to mark their own torrents as trumpable";

/// User 164, from the `with_test_users` fixture, who holds the `edit_torrent` and
/// `edit_torrent_trumpable` permissions.
const EDIT_TRUMPABLE_USER_ID: i32 = 164;

/// Lists the conversations the authenticated user takes part in, newest first.
const SEARCH_CONVERSATIONS_URI: &str = "/api/search/conversations?page=1&page_size=50&search_titles_only=true&order_by_column=last_message&order_by_direction=desc&all_conversations=false";

// Builds an EditedTorrent payload for a torrent, overriding the trumpable field.
fn edit_torrent_payload(torrent_id: i32, release_name: &str, trumpable: &str) -> serde_json::Value {
    serde_json::json!({
        "id": torrent_id,
        "edition_group_id": 1,
        "extras": [],
        "release_name": release_name,
        "release_group": "",
        "uploaded_as_anonymous": false,
        "container": "FLAC",
        "languages": [],
        "features": [],
        "subtitle_languages": [],
        "trumpable": trumpable,
        "bonus_points_snatch_cost": 0
    })
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
async fn test_edit_torrent_trumpable_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditTorrentAndTrumpable,
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/torrents")
        .insert_header(auth_header(&user.token))
        .set_json(edit_torrent_payload(
            TORRENT_ID,
            TORRENT_RELEASE_NAME,
            "a new trump reason",
        ))
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::OK,
    )
    .await;

    let torrent = pool.find_torrent(TORRENT_ID).await.unwrap();
    assert_eq!(torrent.trumpable.as_deref(), Some("a new trump reason"));
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
async fn test_edit_torrent_trumpable_without_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditTorrent,
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/torrents")
        .insert_header(auth_header(&user.token))
        .set_json(edit_torrent_payload(
            TORRENT_ID,
            TORRENT_RELEASE_NAME,
            "an unauthorized trump reason",
        ))
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::FORBIDDEN,
    )
    .await;

    // the trumpable field must remain unchanged
    let torrent = pool.find_torrent(TORRENT_ID).await.unwrap();
    assert_eq!(torrent.trumpable.as_deref(), Some(""));
}

// Editing other fields without the trumpable permission is allowed,
// as long as the trumpable field itself is not changed.
#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../storage/migrations"
)]
async fn test_edit_torrent_without_changing_trumpable(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditTorrent,
    )
    .await;

    // keep trumpable at its unchanged fixture value (empty string)
    let mut payload = edit_torrent_payload(TORRENT_ID, TORRENT_RELEASE_NAME, "");
    payload["release_group"] = serde_json::json!("a new release group");

    let req = test::TestRequest::put()
        .uri("/api/torrents")
        .insert_header(auth_header(&user.token))
        .set_json(payload)
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::OK,
    )
    .await;

    let torrent = pool.find_torrent(TORRENT_ID).await.unwrap();
    assert_eq!(
        torrent.release_group.as_deref(),
        Some("a new release group")
    );
}

/// Marking a torrent as trumpable after its upload sends a message to its uploader,
/// followed by the `automated_message_on_torrent_marked_trumpable` setting.
#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrents_of_basic_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_marking_a_torrent_as_trumpable_notifies_its_uploader(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, editor) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditTorrentAndTrumpable,
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/torrents")
        .insert_header(auth_header(&editor.token))
        .set_json(edit_torrent_payload(
            TORRENT_OF_BASIC_USER_ID,
            TORRENT_OF_BASIC_USER_RELEASE_NAME,
            "a new trump reason",
        ))
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::OK,
    )
    .await;

    // the uploader of the torrent receives the message, sent by the user who marked it trumpable
    let uploader = common::login_as(&service, TestUser::Standard).await;
    let req = test::TestRequest::get()
        .uri(SEARCH_CONVERSATIONS_URI)
        .insert_header(auth_header(&uploader.token))
        .to_request();
    let results: PaginatedResults<ConversationSearchResult> =
        common::call_and_read_body_json(&service, req).await;

    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].sender_id, EDIT_TRUMPABLE_USER_ID);

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/conversations?id={}",
            results.results[0].conversation_id
        ))
        .insert_header(auth_header(&uploader.token))
        .to_request();
    let conversation: ConversationHierarchy = common::call_and_read_body_json(&service, req).await;

    let message_footer = pool
        .get_arcadia_settings()
        .await
        .unwrap()
        .automated_message_on_torrent_marked_trumpable
        .unwrap();
    assert_eq!(
        conversation.messages[0].content,
        format!("Your torrent [url=/torrent/900]Love Me Do / P.S. I Love You[/url] has been marked as trumpable for the following reason: a new trump reason.\n{message_footer}")
    );
}

/// Saving the very same trumpable reason again is not a new mark, so it sends no second message.
#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrents_of_basic_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_editing_an_existing_trumpable_reason_sends_no_message(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, editor) = common::create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditTorrentAndTrumpable,
    )
    .await;

    for _ in 0..2 {
        let req = test::TestRequest::put()
            .uri("/api/torrents")
            .insert_header(auth_header(&editor.token))
            .set_json(edit_torrent_payload(
                TORRENT_OF_BASIC_USER_ID,
                TORRENT_OF_BASIC_USER_RELEASE_NAME,
                "a new trump reason",
            ))
            .to_request();

        common::call_and_read_body_json_with_status::<serde_json::Value, _>(
            &service,
            req,
            StatusCode::OK,
        )
        .await;
    }

    // only the edit that actually set the reason notified the uploader
    let uploader = common::login_as(&service, TestUser::Standard).await;
    let req = test::TestRequest::get()
        .uri(SEARCH_CONVERSATIONS_URI)
        .insert_header(auth_header(&uploader.token))
        .to_request();
    let results: PaginatedResults<ConversationSearchResult> =
        common::call_and_read_body_json(&service, req).await;

    assert_eq!(results.results.len(), 1);
}

/// The uploader of a torrent needs no notification of their own action.
#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_of_edit_trump_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_marking_own_torrent_as_trumpable_sends_no_message(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, editor) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditTorrentAndTrumpable,
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/torrents")
        .insert_header(auth_header(&editor.token))
        .set_json(edit_torrent_payload(
            TORRENT_OF_EDIT_TRUMP_USER_ID,
            TORRENT_OF_EDIT_TRUMP_USER_RELEASE_NAME,
            "a new trump reason",
        ))
        .to_request();

    common::call_and_read_body_json_with_status::<serde_json::Value, _>(
        &service,
        req,
        StatusCode::OK,
    )
    .await;

    // the torrent is marked as trumpable...
    let torrent = pool
        .find_torrent(TORRENT_OF_EDIT_TRUMP_USER_ID)
        .await
        .unwrap();
    assert_eq!(torrent.trumpable.as_deref(), Some("a new trump reason"));

    // ... but its uploader is not notified about it
    let req = test::TestRequest::get()
        .uri(SEARCH_CONVERSATIONS_URI)
        .insert_header(auth_header(&editor.token))
        .to_request();
    let results: PaginatedResults<ConversationSearchResult> =
        common::call_and_read_body_json(&service, req).await;

    assert!(results.results.is_empty());
}

/// The uploader of the anonymous torrent of the `with_test_anonymous_torrent_of_basic_user`
/// fixture.
const ANONYMOUS_UPLOADER_ID: i32 = 100;

fn search_uploads_of_anonymous_uploader_request(token: &str) -> Request {
    test::TestRequest::get()
        .uri(&format!("/api/search/torrents/lite?torrent_created_by_id={ANONYMOUS_UPLOADER_ID}&page=1&page_size=5&order_by_column=torrent_created_at&order_by_direction=desc&title_group_include_empty_groups=false"))
        .insert_header(auth_header(token))
        .to_request()
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_anonymous_torrent_of_basic_user",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_searching_the_torrents_uploaded_anonymously_by_a_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, uploader) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // the uploader finds their own anonymous uploads, marked as anonymous
    let results: PaginatedResults<TitleGroupHierarchyLite> = common::call_and_read_body_json(
        &service,
        search_uploads_of_anonymous_uploader_request(&uploader.token),
    )
    .await;
    let torrent = &results.results[0].edition_groups[0].torrents[0];
    assert!(torrent.uploaded_as_anonymous);
    assert_eq!(
        torrent.created_by.as_ref().map(|user| user.id),
        Some(ANONYMOUS_UPLOADER_ID)
    );

    // another user does not get them at all
    let other_user = common::login_as(&service, TestUser::EditArtist).await;
    let results: PaginatedResults<TitleGroupHierarchyLite> = common::call_and_read_body_json(
        &service,
        search_uploads_of_anonymous_uploader_request(&other_user.token),
    )
    .await;
    assert!(results.results.is_empty());

    // the users allowed to see the information hidden by the paranoia settings get them,
    // together with the name of the uploader and the fact that the upload is anonymous
    let staff_user = common::login_as(&service, TestUser::SeeParanoiaHiddenUserInfo).await;
    let results: PaginatedResults<TitleGroupHierarchyLite> = common::call_and_read_body_json(
        &service,
        search_uploads_of_anonymous_uploader_request(&staff_user.token),
    )
    .await;
    let torrent = &results.results[0].edition_groups[0].torrents[0];
    assert!(torrent.uploaded_as_anonymous);
    assert_eq!(
        torrent.created_by.as_ref().map(|user| user.id),
        Some(ANONYMOUS_UPLOADER_ID)
    );
}

/// The user snatching the anonymous torrent in the
/// `with_test_snatch_of_the_anonymous_torrent_by_the_edit_artist_user` fixture.
const SNATCHING_USER_ID: i32 = 101;

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_anonymous_torrent_of_basic_user",
        "with_refreshed_title_group_hierarchy_lite",
        "with_test_snatch_of_the_anonymous_torrent_by_the_edit_artist_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_the_uploader_of_a_snatched_anonymous_torrent_stays_hidden(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, snatching_user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditArtist)
            .await;

    // a user seeing their own profile is allowed to see the information hidden by the paranoia
    // settings, but that must not reveal the uploader of the torrents they snatched
    let profile: PublicProfile = common::call_and_read_body_json(
        &service,
        test::TestRequest::get()
            .uri(&format!("/api/users?id={SNATCHING_USER_ID}"))
            .insert_header(auth_header(&snatching_user.token))
            .to_request(),
    )
    .await;
    let torrent = &profile.last_five_snatched_torrents[0].edition_groups[0].torrents[0];
    assert!(torrent.uploaded_as_anonymous);
    assert!(torrent.created_by.is_none());
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_uploads_disabled"
    ),
    migrations = "../storage/migrations"
)]
async fn test_cannot_create_title_group_edition_group_or_torrent_when_uploads_disabled(
    pool: PgPool,
) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let title_group_request = test::TestRequest::post()
        .uri("/api/title-groups")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({
            "name": "Abbey Road",
            "name_aliases": [],
            "description": "A title group created while uploads are disabled",
            "original_language": null,
            "country_from": null,
            "covers": [],
            "external_links": [],
            "trailers": [],
            "category": null,
            "content_type": "music",
            "tags": ["rock"],
            "tagline": null,
            "platform": null,
            "original_release_date": null,
            "original_release_date_only_year_known": false,
            "affiliated_artists": [],
            "series_id": null,
            "screenshots": [],
            "master_group_id": null
        }))
        .to_request();

    let edition_group_request = test::TestRequest::post()
        .uri("/api/edition-groups")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({
            "name": null,
            "release_date": null,
            "release_date_only_year_known": false,
            "description": null,
            "distributor": null,
            "covers": [],
            "external_links": [],
            "source": null,
            "additional_information": null,
            "title_group_id": 1
        }))
        .to_request();

    let torrent_request = build_test_torrent_upload_request(&user.token, "TESTGROUP").await;

    for request in [title_group_request, edition_group_request, torrent_request] {
        let path = request.path().to_string();
        let response = test::call_service(&service, request).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{path}");
    }
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrents_of_basic_user",
        "with_test_snatch_of_basic_user",
        "with_test_artist",
        "with_test_affiliated_artist",
        "with_refreshed_title_group_hierarchy_lite"
    ),
    migrations = "../storage/migrations"
)]
async fn test_torrent_search_and_user_activities_include_affiliated_artists(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/torrents/lite?torrent_snatched_by_id=100&page=1&page_size=5&order_by_column=torrent_snatched_at&order_by_direction=desc&title_group_include_empty_groups=false")
        .insert_header(auth_header(&user.token))
        .to_request();
    let search: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(search.results[0].affiliated_artists.0.len(), 1);
    assert_eq!(
        search.results[0].affiliated_artists.0[0].name,
        "The Beatles"
    );

    let req = test::TestRequest::get()
        .uri("/api/users/torrent-activities?page=1&page_size=5&include_unseeded_torrents=true&order_by_column=grabbed_at&order_by_direction=desc&hours_seeding_per_day=24&seeders_per_torrent=current")
        .insert_header(auth_header(&user.token))
        .to_request();
    let activities: PaginatedResults<TorrentActivityAndTitleGroup> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let affiliated_artists = &activities.results[0].title_group.affiliated_artists.0;
    assert_eq!(affiliated_artists.len(), 1);
    assert_eq!(affiliated_artists[0].name, "The Beatles");
}

#[actix_web::test]
async fn test_torrent_with_out_of_range_piece_length_is_rejected() {
    // the piece length sizes allocations when the torrent is rebuilt, a huge one used to abort the process
    let torrent_with_piece_length = |piece_length: &str| {
        format!(
            "d4:infod6:lengthi1e4:name4:file12:piece lengthi{piece_length}e6:pieces20:{}ee",
            "a".repeat(20)
        )
    };

    assert!(bip_metainfo::Metainfo::from_bytes(torrent_with_piece_length("16384")).is_ok());
    for piece_length in ["4611686018427387904", "0", "-1"] {
        assert!(
            bip_metainfo::Metainfo::from_bytes(torrent_with_piece_length(piece_length)).is_err()
        );
    }
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrents_of_basic_user"
    ),
    migrations = "../storage/migrations"
)]
async fn test_uploader_can_not_change_the_snatch_cost_when_the_settings_forbid_it(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = common::create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let mut payload = edit_torrent_payload(
        TORRENT_OF_BASIC_USER_ID,
        TORRENT_OF_BASIC_USER_RELEASE_NAME,
        "false",
    );
    payload["bonus_points_snatch_cost"] = 1000000.into();
    let req = test::TestRequest::put()
        .uri("/api/torrents")
        .insert_header(auth_header(&user.token))
        .set_json(payload)
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let torrent = pool.find_torrent(TORRENT_OF_BASIC_USER_ID).await.unwrap();
    assert_eq!(torrent.bonus_points_snatch_cost, 0);
}
