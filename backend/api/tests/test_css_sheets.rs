use crate::{common, mocks};

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::css_sheet::{CssSheet, CssSheetsEnriched, EditedCssSheet, UserCreatedCssSheet},
};
use common::{
    TestUser, auth_header, call_and_read_body_json, call_and_read_body_json_with_status,
    create_test_app, create_test_app_and_login,
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_can_create_css_sheet(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateCssSheet).await;

    let css_sheet = UserCreatedCssSheet {
        name: "test_sheet".into(),
        css: "body { color: red; }".into(),
        preview_image_url: "https://example.com/preview.png".into(),
    };

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/css-sheets")
        .set_json(&css_sheet)
        .to_request();

    let created =
        call_and_read_body_json_with_status::<CssSheet, _>(&service, req, StatusCode::CREATED)
            .await;

    assert_eq!(created.name, "test_sheet");
    assert_eq!(created.css, "body { color: red; }");
    assert_eq!(created.preview_image_url, "https://example.com/preview.png");
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_regular_user_cannot_create_css_sheet(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let css_sheet = UserCreatedCssSheet {
        name: "test_sheet".into(),
        css: "body { color: red; }".into(),
        preview_image_url: "https://example.com/preview.png".into(),
    };

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/css-sheets")
        .set_json(&css_sheet)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_css_sheets"),
    migrations = "../storage/migrations"
)]
async fn test_get_css_sheets(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/css-sheets")
        .to_request();

    let sheets = call_and_read_body_json::<CssSheetsEnriched, _>(&service, req).await;

    assert!(!sheets.css_sheets.is_empty());
    assert!(!sheets.default_sheet_name.is_empty());
    // Verify fixture sheets are present
    assert!(sheets.css_sheets.iter().any(|s| s.name == "test_sheet_1"));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_css_sheets"),
    migrations = "../storage/migrations"
)]
async fn test_get_css_sheet_by_name(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/css-sheets/test_sheet_1")
        .to_request();

    let sheet = call_and_read_body_json::<CssSheet, _>(&service, req).await;

    assert_eq!(sheet.name, "test_sheet_1");
    assert_eq!(sheet.css, "body { color: red; }");
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_get_nonexistent_css_sheet(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/css-sheets/nonexistent")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_css_sheets"),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_edit_css_sheet(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditCssSheet).await;

    let edited = EditedCssSheet {
        old_name: "test_sheet_1".into(),
        name: "updated_sheet".into(),
        css: "body { color: blue; }".into(),
        preview_image_url: "https://example.com/new_preview.png".into(),
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/css-sheets")
        .set_json(&edited)
        .to_request();

    let updated = call_and_read_body_json::<CssSheet, _>(&service, req).await;

    assert_eq!(updated.name, "updated_sheet");
    assert_eq!(updated.css, "body { color: blue; }");
    assert_eq!(
        updated.preview_image_url,
        "https://example.com/new_preview.png"
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_css_sheets"),
    migrations = "../storage/migrations"
)]
async fn test_regular_user_cannot_edit_css_sheet(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let edited = EditedCssSheet {
        old_name: "test_sheet_1".into(),
        name: "updated_sheet".into(),
        css: "body { color: blue; }".into(),
        preview_image_url: "https://example.com/new_preview.png".into(),
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/css-sheets")
        .set_json(&edited)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_css_sheets"),
    migrations = "../storage/migrations"
)]
async fn test_get_css_sheet_content_public(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;

    // Get CSS content (public endpoint, no auth required)
    let req = test::TestRequest::get()
        .uri("/api/css/test_sheet_1.css")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let css_content = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(css_content, "body { color: red; }");
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_get_nonexistent_css_sheet_content(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;

    let req = test::TestRequest::get()
        .uri("/css/nonexistent.css")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_css_sheet_endpoints_require_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;

    // Test GET /api/css-sheets requires auth
    let req = test::TestRequest::get().uri("/api/css-sheets").to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Test GET /api/css-sheets/{name} requires auth
    let req = test::TestRequest::get()
        .uri("/api/css-sheets/test_sheet_1")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_css_sheets"),
    migrations = "../storage/migrations"
)]
async fn test_edit_default_css_sheet_name_updates_default(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditCssSheet).await;

    // Edit the default CSS sheet name (arcadia is the default from migration)
    let edited = EditedCssSheet {
        old_name: "arcadia".into(),
        name: "arcadia_updated".into(),
        css: "body { color: purple; }".into(),
        preview_image_url: "https://example.com/new_preview.png".into(),
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/css-sheets")
        .set_json(&edited)
        .to_request();

    let updated = call_and_read_body_json::<CssSheet, _>(&service, req).await;
    assert_eq!(updated.name, "arcadia_updated");

    // Get the CSS sheets list and verify the default name is updated
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/css-sheets")
        .to_request();

    let sheets = call_and_read_body_json::<CssSheetsEnriched, _>(&service, req).await;
    assert_eq!(sheets.default_sheet_name, "arcadia_updated");
}
