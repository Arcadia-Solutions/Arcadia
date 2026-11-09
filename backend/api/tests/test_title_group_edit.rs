use crate::common::{
    TestUser, authenticated_request, create_test_app, create_test_app_and_login, login_as,
};
use crate::mocks::mock_redis::MockRedisPool;
use actix_web::http::{Method, StatusCode};
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

fn edited_title_group(name: &str) -> Option<Value> {
    Some(json!({
        "id": 1,
        "master_group_id": null,
        "name": name,
        "name_aliases": [],
        "description": "description",
        "platform": null,
        "original_language": null,
        "original_release_date": null,
        "original_release_date_only_year_known": false,
        "tagline": null,
        "country_from": null,
        "covers": [],
        "external_links": [],
        "trailers": [],
        "category": null,
        "content_type": "music",
        "screenshots": [],
    }))
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_title_group_can_only_be_edited_by_its_creator_or_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(Arc::clone(&pool), MockRedisPool::default()).await;
    let standard_user = login_as(&service, TestUser::Standard).await;
    let editor = login_as(&service, TestUser::EditTitleGroup).await;

    let request = authenticated_request(
        Method::PUT,
        "/api/title-groups",
        &standard_user.token,
        edited_title_group("vandalized"),
    );
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let request = authenticated_request(
        Method::PUT,
        "/api/title-groups",
        &editor.token,
        edited_title_group("edited by staff"),
    );
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        pool.find_title_group(1).await.unwrap().name,
        "edited by staff"
    );
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_title_group_created_by_basic_user"
))]
async fn test_title_group_can_be_edited_by_its_creator(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let request = authenticated_request(
        Method::PUT,
        "/api/title-groups",
        &user.token,
        edited_title_group("edited by creator"),
    );
    let response = test::call_service(&service, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        pool.find_title_group(1).await.unwrap().name,
        "edited by creator"
    );
}
