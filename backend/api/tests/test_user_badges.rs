pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::user_badge::{
    UserBadgeCategory, UserBadgeManualAward, UserCreatedUserBadgeCategory,
};
use common::{TestUser, auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_create_category_requires_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let body = UserCreatedUserBadgeCategory {
        name: "achievements".into(),
    };

    let req = test::TestRequest::post()
        .uri("/api/user-badge-categories")
        .insert_header(auth_header(&user.token))
        .set_json(&body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_can_create_category(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateUserBadge).await;

    let body = UserCreatedUserBadgeCategory {
        name: "achievements".into(),
    };

    let req = test::TestRequest::post()
        .uri("/api/user-badge-categories")
        .insert_header(auth_header(&staff.token))
        .set_json(&body)
        .to_request();

    let category: UserBadgeCategory =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    assert_eq!(category.name, "achievements");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_badges"),
    migrations = "../storage/migrations"
)]
async fn test_list_badges_redacts_secret_for_non_privileged_viewer(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/user-badges")
        .insert_header(auth_header(&user.token))
        .to_request();

    let badges: Vec<Value> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let visible = badges.iter().find(|b| b["id"] == 600).unwrap();
    assert_eq!(visible["name"], "visible_manual");

    let secret = badges.iter().find(|b| b["id"] == 601).unwrap();
    assert!(secret.get("name").is_none());
    assert_eq!(secret["is_secret"], true);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_badges"),
    migrations = "../storage/migrations"
)]
async fn test_list_badges_full_for_privileged_viewer(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, viewer) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ViewInvisibleUserBadges,
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/user-badges")
        .insert_header(auth_header(&viewer.token))
        .to_request();

    let badges: Vec<Value> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let secret = badges.iter().find(|b| b["id"] == 601).unwrap();
    assert_eq!(secret["name"], "secret_manual");
    assert_eq!(secret["is_secret"], true);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_badges"),
    migrations = "../storage/migrations"
)]
async fn test_user_profile_includes_secret_earned_badges(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/users?id=100")
        .insert_header(auth_header(&user.token))
        .to_request();

    let profile: Value =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let earned = profile["earned_badges"].as_array().unwrap();
    let secret = earned.iter().find(|e| e["badge_id"] == 601).unwrap();
    assert_eq!(secret["badge_name"], "secret_manual");
    assert_eq!(secret["badge_is_secret"], true);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_badges"),
    migrations = "../storage/migrations"
)]
async fn test_award_badge_requires_award_user_badge_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateUserBadge).await;

    let award = UserBadgeManualAward {
        user_id: 101,
        badge_id: 600,
        note: None,
    };
    let req = test::TestRequest::post()
        .uri("/api/user-badges/award")
        .insert_header(auth_header(&staff.token))
        .set_json(&award)
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_badges"),
    migrations = "../storage/migrations"
)]
async fn test_award_badge_conflict_on_duplicate(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::AwardUserBadge).await;

    let award = UserBadgeManualAward {
        user_id: 101,
        badge_id: 600,
        note: Some("nice work".into()),
    };

    let req = test::TestRequest::post()
        .uri("/api/user-badges/award")
        .insert_header(auth_header(&staff.token))
        .set_json(&award)
        .to_request();
    let earned: Value =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    assert_eq!(earned["badge_id"], 600);

    let req2 = test::TestRequest::post()
        .uri("/api/user-badges/award")
        .insert_header(auth_header(&staff.token))
        .set_json(&award)
        .to_request();
    let resp2 = test::call_service(&service, req2).await;
    assert_eq!(resp2.status(), StatusCode::CONFLICT);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_badges"),
    migrations = "../storage/migrations"
)]
async fn test_revoke_requires_revoke_user_badge_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::AwardUserBadge).await;

    let req = test::TestRequest::delete()
        .uri("/api/user-badges/award/1")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_badges"),
    migrations = "../storage/migrations"
)]
async fn test_revoke_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::RevokeUserBadge).await;

    // Earned badge id 1 is the secret manual award seeded by the fixture (user 100, badge 601).
    let req = test::TestRequest::delete()
        .uri("/api/user-badges/award/1")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
