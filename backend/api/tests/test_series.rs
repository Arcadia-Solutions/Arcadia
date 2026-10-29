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
        series::{EditedSeries, Series},
        title_group::TitleGroupAndAssociatedData,
    },
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{
    auth_header, call_and_read_body_json_with_status, create_test_app_and_login, TestUser,
};

#[sqlx::test(
    fixtures("with_test_users", "with_test_series"),
    migrations = "../storage/migrations"
)]
async fn test_edit_series(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditSeries).await;

    let payload = EditedSeries {
        id: 1,
        name: "Updated Series".to_string(),
        description: "Updated description".to_string(),
        covers: vec!["https://example.com/updated-cover.jpg".to_string()],
        banners: vec!["https://example.com/updated-banner.jpg".to_string()],
        tags: vec!["updated".to_string()],
    };

    let req = test::TestRequest::put()
        .uri("/api/series")
        .insert_header(auth_header(&user.token))
        .set_json(&payload)
        .to_request();

    let series: Series = call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(series.name, payload.name);
    assert_eq!(series.description, payload.description);
    assert_eq!(series.covers, payload.covers);
    assert_eq!(series.banners, Some(payload.banners));
    assert_eq!(series.tags, payload.tags);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_series", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_add_title_group_to_series(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (standard_service, standard_user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditTitleGroup).await;

    // title group 1 was created by another user, so only a title group editor can add it
    for (service, user, expected_status) in [
        (&standard_service, &standard_user, StatusCode::FORBIDDEN),
        (&service, &user, StatusCode::OK),
    ] {
        let req = test::TestRequest::post()
            .uri("/api/series/title-group")
            .insert_header(auth_header(&user.token))
            .set_json(serde_json::json!({
                "series_id": 1,
                "title_group_id": 1
            }))
            .to_request();
        assert_eq!(call_service(service, req).await.status(), expected_status);
    }

    let req = test::TestRequest::get()
        .uri("/api/title-groups?id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let title_group: TitleGroupAndAssociatedData =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(title_group.title_group.id, 1);
    assert_eq!(title_group.title_group.series_id, Some(1));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_series", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_remove_title_group_from_series_without_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (editor_service, editor) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditTitleGroup,
    )
    .await;
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // First assign the title group to the series
    let req = test::TestRequest::post()
        .uri("/api/series/title-group")
        .insert_header(auth_header(&editor.token))
        .set_json(serde_json::json!({
            "series_id": 1,
            "title_group_id": 1
        }))
        .to_request();
    let _ = call_service(&editor_service, req).await;

    // Try to remove without permission
    let req = test::TestRequest::delete()
        .uri("/api/series/title-group")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({
            "series_id": 1,
            "title_group_id": 1
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Verify the title group is still assigned to the series
    let req = test::TestRequest::get()
        .uri("/api/title-groups?id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let title_group: TitleGroupAndAssociatedData =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(title_group.title_group.series_id, Some(1));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_series", "with_test_title_group"),
    migrations = "../storage/migrations"
)]
async fn test_remove_title_group_from_series_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::RemoveTitleGroupFromSeries,
    )
    .await;

    // First assign the title group to the series
    let req = test::TestRequest::post()
        .uri("/api/series/title-group")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({
            "series_id": 1,
            "title_group_id": 1
        }))
        .to_request();
    let _ = call_service(&service, req).await;

    // Now remove with the correct permission
    let req = test::TestRequest::delete()
        .uri("/api/series/title-group")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({
            "series_id": 1,
            "title_group_id": 1
        }))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the title group is no longer assigned to the series
    let title_group = pool.find_title_group(1).await.unwrap();
    assert_eq!(title_group.series_id, None);
}
