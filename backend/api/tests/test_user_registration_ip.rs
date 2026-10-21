pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::ip_check::{
    IpCheck, IpCheckProvider, IpCheckResult, UserRegistrationIp,
};
use common::{auth_header, create_test_app_and_login, TestUser};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

const BASIC_USER_ID: i32 = 100;

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_registration_ip_check"),
    migrations = "../storage/migrations"
)]
async fn test_get_user_registration_ip_with_its_check(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ViewRegistrationIps,
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{BASIC_USER_ID}/registration-ip"))
        .insert_header(auth_header(&user.token))
        .to_request();

    let registration_ip: UserRegistrationIp = common::call_and_read_body_json(&service, req).await;

    assert_eq!(
        registration_ip.registered_from_ip.ip().to_string(),
        "10.10.4.88"
    );
    assert_eq!(registration_ip.provider, Some(IpCheckProvider::Proxycheck));
    assert_eq!(registration_ip.city.as_deref(), Some("Paris"));
    assert_eq!(registration_ip.is_vpn, Some(true));
    assert_eq!(registration_ip.is_hosting, None);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_set_user_registration_ip_check(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ViewRegistrationIps,
    )
    .await;

    let check = IpCheck {
        provider: IpCheckProvider::Vpnapi,
        result: IpCheckResult {
            city: Some("Berlin".into()),
            is_tor: Some(true),
            ..Default::default()
        },
    };
    let req = test::TestRequest::put()
        .uri(&format!("/api/users/{BASIC_USER_ID}/registration-ip"))
        .insert_header(auth_header(&user.token))
        .set_json(&check)
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let registration_ip = pool.find_user_registration_ip(BASIC_USER_ID).await.unwrap();
    assert_eq!(registration_ip.provider, Some(IpCheckProvider::Vpnapi));
    assert_eq!(registration_ip.city.as_deref(), Some("Berlin"));
    assert_eq!(registration_ip.is_tor, Some(true));
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_check_private_user_registration_ip(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ViewRegistrationIps,
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{BASIC_USER_ID}/registration-ip/check"))
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    // the user registered from 10.10.4.88, which providers cannot tell anything about
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
