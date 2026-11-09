use crate::{common, mocks};

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use common::{auth_header, create_test_app_and_login, login_as, read_body_json_data};
use mocks::mock_redis::MockRedisPool;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(fixtures("with_test_users", "with_similar_title_groups"))]
async fn test_link_similar_title_groups_appears_in_get_title_group(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::LinkSimilarTitleGroup,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/title-groups/similar")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"group_1": 20, "group_2": 21, "note": "same director"}))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::get()
        .uri("/api/title-groups?id=20")
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    let body: Value = read_body_json_data(resp).await;

    let similar = body["similar_title_groups"].as_array().unwrap();
    assert_eq!(similar.len(), 1);
    assert_eq!(similar[0]["id"], 21);
    assert_eq!(similar[0]["name"], "Second Similar Title Group");
    assert_eq!(similar[0]["original_release_date"], "2002-02-02");
    assert_eq!(similar[0]["cover"], "https://cover-21-a.example.com");
    assert_eq!(similar[0]["note"], "same director");
}

#[sqlx::test(fixtures("with_test_users", "with_similar_title_groups"))]
async fn test_user_without_permission_cannot_unlink_similar_title_groups(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::LinkSimilarTitleGroup,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/title-groups/similar")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"group_1": 20, "group_2": 21, "note": null}))
        .to_request();
    assert_eq!(
        test::call_service(&service, req).await.status(),
        StatusCode::OK
    );

    let req = test::TestRequest::delete()
        .uri("/api/title-groups/similar")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"group_1": 20, "group_2": 21, "note": null}))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    assert_eq!(pool.find_similar_title_groups(20).await.unwrap().len(), 1);
}

#[sqlx::test(fixtures("with_test_users", "with_similar_title_groups"))]
async fn test_user_with_permission_can_unlink_similar_title_groups(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, linker) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::LinkSimilarTitleGroup,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/title-groups/similar")
        .insert_header(auth_header(&linker.token))
        .set_json(json!({"group_1": 21, "group_2": 20, "note": null}))
        .to_request();
    assert_eq!(
        test::call_service(&service, req).await.status(),
        StatusCode::OK
    );

    let remover = login_as(&service, TestUser::UnlinkSimilarTitleGroup).await;
    let req = test::TestRequest::delete()
        .uri("/api/title-groups/similar")
        .insert_header(auth_header(&remover.token))
        .set_json(json!({"group_1": 20, "group_2": 21, "note": null}))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    assert!(pool.find_similar_title_groups(20).await.unwrap().is_empty());
}

#[sqlx::test(fixtures("with_test_users", "with_similar_title_groups"))]
async fn test_user_without_permission_cannot_link_similar_title_groups(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .uri("/api/title-groups/similar")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"group_1": 20, "group_2": 21, "note": null}))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    assert!(pool.find_similar_title_groups(20).await.unwrap().is_empty());
}
