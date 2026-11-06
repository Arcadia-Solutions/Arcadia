pub mod common;
pub mod mocks;

use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use common::{Profile, TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_get_me_for_regular_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/me")
        .to_request();

    let profile = call_and_read_body_json::<Profile, _>(&service, req).await;

    assert_eq!(profile.user.username, "user_basic");
}
