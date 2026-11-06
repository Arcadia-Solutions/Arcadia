use crate::{common, mocks};

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
