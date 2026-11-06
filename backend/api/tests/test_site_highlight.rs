use crate::mocks;

use std::sync::Arc;

use actix_web::{
    http::StatusCode,
    test::{self, call_service},
};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::site_highlight::{
        CreateSiteHighlight, EditSiteHighlight, SiteHighlight, SiteHighlightItemType,
    },
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;

use crate::common::{
    TestUser, auth_header, call_and_read_body_json_with_status, create_test_app_and_login,
};

fn default_create_payload() -> CreateSiteHighlight {
    CreateSiteHighlight {
        alias: "Series of the week".to_string(),
        item_type: SiteHighlightItemType::Series,
        item_id: 1,
        forum_thread_id: 1,
        enabled: true,
        position: 0,
    }
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_series"),
    migrations = "../storage/migrations"
)]
async fn test_create_site_highlight_inserts_related_thread(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageSiteHighlights,
    )
    .await;

    let payload = default_create_payload();

    let req = test::TestRequest::post()
        .uri("/api/site-highlights")
        .insert_header(auth_header(&user.token))
        .set_json(&payload)
        .to_request();

    let created: SiteHighlight =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(created.alias, "Series of the week");
    assert_eq!(created.item_type, SiteHighlightItemType::Series);
    assert_eq!(created.item_id, 1);
    assert_eq!(created.forum_thread_id, 1);
    assert!(created.enabled);

    let related = pool.find_related_forum_threads_for_series(1).await.unwrap();
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].forum_thread_id, 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_series"),
    migrations = "../storage/migrations"
)]
async fn test_delete_site_highlight_keeps_related_thread(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageSiteHighlights,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/site-highlights")
        .insert_header(auth_header(&user.token))
        .set_json(default_create_payload())
        .to_request();
    let created: SiteHighlight =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/site-highlights/{}", created.id))
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let related = pool.find_related_forum_threads_for_series(1).await.unwrap();
    assert_eq!(related.len(), 1);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_create_site_highlight_without_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .uri("/api/site-highlights")
        .insert_header(auth_header(&user.token))
        .set_json(default_create_payload())
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_series",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread"
    ),
    migrations = "../storage/migrations"
)]
async fn test_edit_site_highlight_keeps_old_related_thread_when_toggle_off(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageSiteHighlights,
    )
    .await;

    let create_payload = CreateSiteHighlight {
        forum_thread_id: 100,
        ..default_create_payload()
    };
    let req = test::TestRequest::post()
        .uri("/api/site-highlights")
        .insert_header(auth_header(&user.token))
        .set_json(&create_payload)
        .to_request();
    let created: SiteHighlight =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let edit_payload = EditSiteHighlight {
        alias: None,
        item_type: None,
        item_id: None,
        forum_thread_id: Some(101),
        enabled: None,
        position: None,
        remove_previous_related_thread: false,
    };
    let req = test::TestRequest::put()
        .uri(&format!("/api/site-highlights/{}", created.id))
        .insert_header(auth_header(&user.token))
        .set_json(&edit_payload)
        .to_request();
    let _edited: SiteHighlight =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let related = pool.find_related_forum_threads_for_series(1).await.unwrap();
    assert_eq!(related.len(), 2);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_series",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread"
    ),
    migrations = "../storage/migrations"
)]
async fn test_edit_site_highlight_removes_old_related_thread_by_default(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageSiteHighlights,
    )
    .await;

    let create_payload = CreateSiteHighlight {
        forum_thread_id: 100,
        ..default_create_payload()
    };
    let req = test::TestRequest::post()
        .uri("/api/site-highlights")
        .insert_header(auth_header(&user.token))
        .set_json(&create_payload)
        .to_request();
    let created: SiteHighlight =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let edit_payload = EditSiteHighlight {
        alias: None,
        item_type: None,
        item_id: None,
        forum_thread_id: Some(101),
        enabled: None,
        position: None,
        remove_previous_related_thread: true,
    };
    let req = test::TestRequest::put()
        .uri(&format!("/api/site-highlights/{}", created.id))
        .insert_header(auth_header(&user.token))
        .set_json(&edit_payload)
        .to_request();
    let _edited: SiteHighlight =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let related = pool.find_related_forum_threads_for_series(1).await.unwrap();
    assert_eq!(related.len(), 1);
    assert_eq!(related[0].forum_thread_id, 101);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_series"),
    migrations = "../storage/migrations"
)]
async fn test_create_site_highlight_position_conflict_returns_409(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ManageSiteHighlights,
    )
    .await;

    let payload = CreateSiteHighlight {
        position: 5,
        ..default_create_payload()
    };
    let req = test::TestRequest::post()
        .uri("/api/site-highlights")
        .insert_header(auth_header(&user.token))
        .set_json(&payload)
        .to_request();
    let _: SiteHighlight = call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let payload = CreateSiteHighlight {
        alias: "Second".to_string(),
        position: 5,
        ..default_create_payload()
    };
    let req = test::TestRequest::post()
        .uri("/api/site-highlights")
        .insert_header(auth_header(&user.token))
        .set_json(&payload)
        .to_request();
    let resp = call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}
