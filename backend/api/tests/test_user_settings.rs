use crate::{common, mocks};

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::css_sheet::UserCreatedCssSheet,
    models::user::{UserSettings, UserSettingsResponse},
};
use common::{
    TestUser, auth_header, call_and_read_body_json, create_test_app, create_test_app_and_login,
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_get_user_settings(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/settings")
        .to_request();

    let _ = call_and_read_body_json::<UserSettingsResponse, _>(&service, req).await;
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_update_user_settings(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // First, create a CSS sheet as staff user
    let (service, staff_user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::CreateCssSheet,
    )
    .await;

    let css_sheet = UserCreatedCssSheet {
        name: "custom_sheet".into(),
        css: "body { color: red; }".into(),
        preview_image_url: "https://example.com/preview.png".into(),
    };

    let create_req = test::TestRequest::post()
        .insert_header(auth_header(&staff_user.token))
        .uri("/api/css-sheets")
        .set_json(&css_sheet)
        .to_request();

    test::call_service(&service, create_req).await;

    // Now test updating user settings as regular user (reuse same pool, create new service)
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let new_settings = UserSettings {
        css_sheet_name: "custom_sheet".into(),
        irc_site_embed_enabled: false,
        show_upload_method: true,
        paranoia_hidden_stats: vec![],
        paranoia_hidden_lists: vec![],
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/settings")
        .set_json(&new_settings)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the settings were updated by fetching them again
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/settings")
        .to_request();

    let updated_settings = call_and_read_body_json::<UserSettingsResponse, _>(&service, req).await;
    assert_eq!(updated_settings.settings.css_sheet_name, "custom_sheet");
    assert!(updated_settings.settings.show_upload_method);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_get_user_settings_requires_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;

    let req = test::TestRequest::get()
        .uri("/api/users/settings")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
