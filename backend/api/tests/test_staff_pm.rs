use crate::{common, mocks};
use actix_web::http::Method;
use common::{authenticated_request, create_test_app, login_as};
use serde_json::Value;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::staff_pm::UserCreatedStaffPmMessage;
use common::{TestUser, auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_staff_pm"),
    migrations = "../storage/migrations"
)]
async fn test_cannot_reply_to_resolved_staff_pm(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::StaffPm).await;

    // Staff PM 100 is resolved
    let message = UserCreatedStaffPmMessage {
        staff_pm_id: 100,
        content: "This should fail".into(),
    };

    let req = test::TestRequest::post()
        .uri("/api/staff-pms/messages")
        .insert_header(auth_header(&user.token))
        .set_json(&message)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_staff_pm"),
    migrations = "../storage/migrations"
)]
async fn test_staff_pm_is_only_visible_to_its_creator_and_staff(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;
    let creator = login_as(&service, TestUser::Standard).await;
    let other_user = login_as(&service, TestUser::EditArtist).await;
    let staff = login_as(&service, TestUser::StaffPm).await;

    for (token, can_see) in [
        (&creator.token, true),
        (&other_user.token, false),
        (&staff.token, true),
    ] {
        let request = authenticated_request(Method::GET, "/api/staff-pms/100", token, None);
        let response = test::call_service(&service, request).await;
        assert_eq!(response.status().is_success(), can_see);

        let request = authenticated_request(Method::GET, "/api/staff-pms", token, None);
        let response = test::call_service(&service, request).await;
        assert_eq!(response.status(), StatusCode::OK);
        let staff_pms: Value = test::read_body_json(response).await;
        let listed = staff_pms["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|staff_pm| staff_pm["id"] == 100);
        assert_eq!(listed, can_see);
    }
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_staff_pm"),
    migrations = "../storage/migrations"
)]
async fn test_staff_pm_can_only_be_unresolved_by_its_creator_or_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;
    let creator = login_as(&service, TestUser::Standard).await;
    // can read and resolve staff PMs, but not unresolve them
    let staff = login_as(&service, TestUser::StaffPm).await;

    let uri = "/api/staff-pms/100/unresolve";
    let request = authenticated_request(Method::PUT, uri, &staff.token, None);
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let request = authenticated_request(Method::PUT, uri, &creator.token, None);
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let staff_pm: Value = test::read_body_json(response).await;
    assert_eq!(staff_pm["data"]["resolved"], false);
}
