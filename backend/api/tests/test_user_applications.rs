use crate::common::{TestUser, authenticated_request, create_test_app_and_login};
use crate::mocks::mock_redis::MockRedisPool;
use actix_web::http::{Method, StatusCode};
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_user_applications_require_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    for (method, body) in [
        (Method::GET, None),
        (
            Method::PUT,
            Some(json!({ "user_application_id": 1, "status": "accepted" })),
        ),
    ] {
        let request =
            authenticated_request(method.clone(), "/api/user-applications", &user.token, body);
        let response = test::call_service(&service, request).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{method}");
    }
}
