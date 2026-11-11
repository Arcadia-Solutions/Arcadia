use crate::{common, mocks};

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::arcadia_settings::{ArcadiaSettings, RatingIcon},
};
use common::{
    TestUser, auth_header, call_and_read_body_json, create_test_app, create_test_app_and_login,
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use sqlx::types::Json;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"))]
async fn test_staff_can_get_arcadia_settings(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .to_request();

    let settings = call_and_read_body_json::<ArcadiaSettings, _>(&service, req).await;

    assert_eq!(settings.user_class_name_on_signup, "newbie");
    assert_eq!(settings.default_css_sheet_name, "arcadia");
    assert!(settings.open_signups);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_regular_user_cannot_get_arcadia_settings(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_get_arcadia_settings_requires_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;

    let req = test::TestRequest::get()
        .uri("/api/arcadia-settings")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_staff_can_update_arcadia_settings(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let updated_settings = ArcadiaSettings {
        user_class_name_on_signup: "newbie".to_string(),
        default_css_sheet_name: "arcadia".to_string(),
        global_upload_factor: 100,
        global_download_factor: 100,
        shop_upload_base_price_per_gb: 100,
        shop_freeleech_token_base_price: 500,
        bonus_points_alias: "bonus points".to_string(),
        invitation_expiration_days: 3,
        reseed_requestable_after_hours: 72,
        ..Default::default()
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .set_json(&updated_settings)
        .to_request();

    let settings = call_and_read_body_json::<ArcadiaSettings, _>(&service, req).await;

    assert_eq!(settings.user_class_name_on_signup, "newbie");
    assert_eq!(settings.default_css_sheet_name, "arcadia");
    assert!(!settings.open_signups);

    // Verify the settings were actually updated in the database
    let db_settings = pool.get_arcadia_settings().await.unwrap();
    assert_eq!(db_settings.user_class_name_on_signup, "newbie");
    assert_eq!(db_settings.default_css_sheet_name, "arcadia");
    assert!(!db_settings.open_signups);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_regular_user_cannot_update_arcadia_settings(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let updated_settings = ArcadiaSettings {
        user_class_name_on_signup: "newbie".to_string(),
        default_css_sheet_name: "arcadia".to_string(),
        open_signups: true,
        global_upload_factor: 100,
        global_download_factor: 100,
        shop_upload_base_price_per_gb: 100,
        shop_freeleech_token_base_price: 500,
        bonus_points_alias: "bonus points".to_string(),
        invitation_expiration_days: 3,
        reseed_requestable_after_hours: 72,
        ..Default::default()
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .set_json(&updated_settings)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_update_arcadia_settings_requires_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;

    let updated_settings = ArcadiaSettings {
        user_class_name_on_signup: "newbie".to_string(),
        default_css_sheet_name: "arcadia".to_string(),
        open_signups: true,
        global_upload_factor: 100,
        global_download_factor: 100,
        shop_upload_base_price_per_gb: 100,
        shop_freeleech_token_base_price: 500,
        bonus_points_alias: "bonus points".to_string(),
        invitation_expiration_days: 3,
        reseed_requestable_after_hours: 72,
        ..Default::default()
    };

    let req = test::TestRequest::put()
        .uri("/api/arcadia-settings")
        .set_json(&updated_settings)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_update_arcadia_settings_updates_in_memory_cache(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    // First, get the current settings
    let get_req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .to_request();

    let initial_settings = call_and_read_body_json::<ArcadiaSettings, _>(&service, get_req).await;
    assert_eq!(initial_settings.user_class_name_on_signup, "newbie");

    // Update the settings
    let updated_settings = ArcadiaSettings {
        user_class_name_on_signup: "newbie".to_string(),
        default_css_sheet_name: "arcadia".to_string(),
        global_upload_factor: 100,
        global_download_factor: 100,
        shop_upload_base_price_per_gb: 100,
        shop_freeleech_token_base_price: 500,
        bonus_points_alias: "bonus points".to_string(),
        invitation_expiration_days: 3,
        reseed_requestable_after_hours: 72,
        ..Default::default()
    };

    let update_req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .set_json(&updated_settings)
        .to_request();

    let updated = call_and_read_body_json::<ArcadiaSettings, _>(&service, update_req).await;
    assert_eq!(updated.user_class_name_on_signup, "newbie");
    assert!(!updated.open_signups);

    // Get the settings again to verify the in-memory cache was updated
    let get_req_after = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .to_request();

    let final_settings =
        call_and_read_body_json::<ArcadiaSettings, _>(&service, get_req_after).await;
    assert_eq!(final_settings.user_class_name_on_signup, "newbie");
    assert_eq!(final_settings.default_css_sheet_name, "arcadia");
    assert!(!final_settings.open_signups);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_update_arcadia_settings_requires_all_automated_message_fields(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let partial_settings = ArcadiaSettings {
        user_class_name_on_signup: "newbie".to_string(),
        default_css_sheet_name: "arcadia".to_string(),
        open_signups: true,
        global_upload_factor: 100,
        global_download_factor: 100,
        automated_message_on_signup: Some("Welcome!".to_string()),
        shop_upload_base_price_per_gb: 100,
        shop_freeleech_token_base_price: 500,
        bonus_points_alias: "bonus points".to_string(),
        invitation_expiration_days: 3,
        reseed_requestable_after_hours: 72,
        ..Default::default()
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .set_json(&partial_settings)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_staff_can_enable_charging_bonus_points_on_resnatch(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    assert!(
        !pool
            .get_arcadia_settings()
            .await
            .unwrap()
            .charge_bonus_points_on_resnatch,
        "Re-snatches should not be charged by default"
    );

    let updated_settings = ArcadiaSettings {
        user_class_name_on_signup: "newbie".to_string(),
        default_css_sheet_name: "arcadia".to_string(),
        bonus_points_alias: "bonus points".to_string(),
        invitation_expiration_days: 3,
        reseed_requestable_after_hours: 72,
        charge_bonus_points_on_resnatch: true,
        ..Default::default()
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .set_json(&updated_settings)
        .to_request();

    let settings = call_and_read_body_json::<ArcadiaSettings, _>(&service, req).await;
    assert!(settings.charge_bonus_points_on_resnatch);

    let db_settings = pool.get_arcadia_settings().await.unwrap();
    assert!(db_settings.charge_bonus_points_on_resnatch);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_staff_cannot_set_invitation_expiration_days_to_zero_or_less(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let updated_settings = ArcadiaSettings {
        user_class_name_on_signup: "newbie".to_string(),
        default_css_sheet_name: "arcadia".to_string(),
        bonus_points_alias: "bonus points".to_string(),
        invitation_expiration_days: 0,
        reseed_requestable_after_hours: 72,
        ..Default::default()
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .set_json(&updated_settings)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_staff_cannot_set_reseed_requestable_after_hours_to_zero(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let updated_settings = ArcadiaSettings {
        user_class_name_on_signup: "newbie".to_string(),
        default_css_sheet_name: "arcadia".to_string(),
        bonus_points_alias: "bonus points".to_string(),
        invitation_expiration_days: 3,
        reseed_requestable_after_hours: 0,
        ..Default::default()
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .set_json(&updated_settings)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures("with_test_users"))]
async fn test_staff_cannot_leave_low_ratings_without_icon(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let updated_settings = ArcadiaSettings {
        user_class_name_on_signup: "newbie".to_string(),
        default_css_sheet_name: "arcadia".to_string(),
        bonus_points_alias: "bonus points".to_string(),
        invitation_expiration_days: 3,
        reseed_requestable_after_hours: 72,
        rating_icons: Json(vec![RatingIcon {
            minimum_rating: 40,
            icon_url: "https://example.com/icon.png".to_string(),
        }]),
        ..Default::default()
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/arcadia-settings")
        .set_json(&updated_settings)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
