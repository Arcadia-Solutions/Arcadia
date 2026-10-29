pub mod common;
pub mod mocks;

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::artist::{
    Artist, ArtistRole, ArtistSearchResult, EditedArtist, UserCreatedAffiliatedArtist,
};
use arcadia_storage::models::common::PaginatedResults;
use common::auth_header;
use common::create_test_app_and_login;
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_artist"),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_edit_artist(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditArtist).await;

    let req_body = EditedArtist{
        id: 1,
        name: "Beatles, The".into(),
        aliases: vec!["Fab Four".into(), "The Fabs".into()],
        description: "They are actually called 'The Beatles', but we decided to be weird with articles.".into(),
        pictures: vec![
            "https://upload.wikimedia.org/wikipedia/commons/d/d8/The_Beatles_members_at_New_York_City_in_1964.jpg".into()
        ],
        external_links: vec![
            "https://en.wikipedia.org/wiki/The_Beatles".into(),
            "https://musicbrainz.org/artist/b10bbbfc-cf9e-42e0-be17-e2c3e1d2600d".into(),
        ],
    };

    let req = test::TestRequest::put()
        .uri("/api/artists")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let response =
        common::call_and_read_body_json_with_status::<Artist, _>(&service, req, StatusCode::OK)
            .await;

    assert_eq!(response.name, req_body.name);
    assert_eq!(response.description, req_body.description);
    assert_eq!(response.pictures.len(), 1);
    assert_eq!(response.pictures[0], req_body.pictures[0]);
    assert_eq!(response.aliases, req_body.aliases);
    assert_eq!(response.external_links, req_body.external_links);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artists_for_search"),
    migrations = "../storage/migrations"
)]
async fn test_search_artists_returns_paginated_results(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/artists?page=1&page_size=10&order_by_column=created_at&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<ArtistSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.results.len(), 4);
    assert_eq!(response.total_items, 4);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 10);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artists_for_search"),
    migrations = "../storage/migrations"
)]
async fn test_search_artists_filters_by_name(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/artists?name=Beatles&page=1&page_size=10&order_by_column=created_at&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<ArtistSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].name, "The Beatles");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artists_for_search"),
    migrations = "../storage/migrations"
)]
async fn test_search_artists_matches_by_alias(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/artists?name=Tea%20Set&page=1&page_size=10&order_by_column=created_at&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<ArtistSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].name, "Pink Floyd");
    assert_eq!(
        response.results[0].aliases,
        vec!["The Pink Floyd Sound".to_string(), "Tea Set".to_string()]
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artists_for_search"),
    migrations = "../storage/migrations"
)]
async fn test_search_artists_accent_insensitive(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/artists?name=gerard&page=1&page_size=10&order_by_column=created_at&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<ArtistSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.total_items, 1);
    assert_eq!(response.results[0].name, "Gérard Depardieu");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artists_for_search"),
    migrations = "../storage/migrations"
)]
async fn test_search_artists_pagination(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/artists?page=1&page_size=2&order_by_column=created_at&order_by_direction=desc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let response: PaginatedResults<ArtistSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.results.len(), 2);
    assert_eq!(response.total_items, 4);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 2);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artist"),
    migrations = "../storage/migrations"
)]
async fn test_user_with_permission_can_delete_artist(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::DeleteArtist,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/artists?artist_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify artist was actually deleted
    let artist_result = pool.find_artist_by_id(1).await;
    assert!(artist_result.is_err());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_artist"),
    migrations = "../storage/migrations"
)]
async fn test_user_without_permission_cannot_delete_artist(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .uri("/api/artists?artist_id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Verify artist was NOT deleted
    let artist = pool.find_artist_by_id(1).await;
    assert!(artist.is_ok());
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_artist",
        "with_test_title_group",
        "with_test_affiliated_artist"
    ),
    migrations = "../storage/migrations"
)]
async fn test_duplicate_artist_affiliation_returns_conflict(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditTitleGroup).await;

    // Artist 1 is already affiliated to title_group 1 via fixture
    let req_body = vec![UserCreatedAffiliatedArtist {
        title_group_id: 1,
        artist_id: 1,
        roles: vec![ArtistRole::Main],
        nickname: None,
    }];

    let req = test::TestRequest::post()
        .uri("/api/affiliated-artists")
        .insert_header(auth_header(&user.token))
        .set_json(&req_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_artist",
        "with_test_title_group",
        "with_test_affiliated_artist"
    ),
    migrations = "../storage/migrations"
)]
async fn test_user_cannot_change_artist_affiliations_of_title_group_created_by_another_user(
    pool: PgPool,
) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // title group 2 was created by another user
    let req = test::TestRequest::post()
        .uri("/api/affiliated-artists")
        .insert_header(auth_header(&user.token))
        .set_json(vec![UserCreatedAffiliatedArtist {
            title_group_id: 2,
            artist_id: 1,
            roles: vec![ArtistRole::Main],
            nickname: None,
        }])
        .to_request();
    assert_eq!(
        test::call_service(&service, req).await.status(),
        StatusCode::FORBIDDEN
    );

    let req = test::TestRequest::delete()
        .uri("/api/affiliated-artists")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({ "affiliation_ids": [1] }))
        .to_request();
    assert_eq!(
        test::call_service(&service, req).await.status(),
        StatusCode::FORBIDDEN
    );
}
