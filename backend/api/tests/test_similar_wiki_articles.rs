pub mod common;
pub mod mocks;

use actix_web::{http::StatusCode, test};
use arcadia_storage::connection_pool::ConnectionPool;
use common::{TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use serde_json::{Value, json};
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_wiki_articles"),
    migrations = "../storage/migrations"
)]
async fn test_user_without_permission_cannot_link_similar_wiki_articles(pool: sqlx::PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .uri("/api/wiki/articles/similar")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"wiki_article_id_1": 1, "wiki_article_id_2": 2}))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_wiki_articles"),
    migrations = "../storage/migrations"
)]
async fn test_link_and_get_similar_wiki_articles(pool: sqlx::PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::LinkSimilarWikiArticles,
    )
    .await;

    // Link articles 1 <-> 2 and 1 <-> 3
    for (a, b) in [(1, 2), (3, 1)] {
        let req = test::TestRequest::post()
            .uri("/api/wiki/articles/similar")
            .insert_header(auth_header(&user.token))
            .set_json(json!({"wiki_article_id_1": a, "wiki_article_id_2": b}))
            .to_request();
        let resp = test::call_service(&service, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // Article 1 should now have articles 2 and 3 as similar
    let req = test::TestRequest::get()
        .uri("/api/wiki/articles?id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let article: Value = call_and_read_body_json(&service, req).await;
    let similar = article["similar_wiki_articles"].as_array().unwrap();
    let mut titles: Vec<&str> = similar
        .iter()
        .map(|a| a["title"].as_str().unwrap())
        .collect();
    titles.sort();
    assert_eq!(titles, vec!["Formatting Guide", "Site Rules"]);

    // Article 2 should reciprocally have article 1 as similar
    let req = test::TestRequest::get()
        .uri("/api/wiki/articles?id=2")
        .insert_header(auth_header(&user.token))
        .to_request();
    let article: Value = call_and_read_body_json(&service, req).await;
    let similar = article["similar_wiki_articles"].as_array().unwrap();
    assert_eq!(similar.len(), 1);
    assert_eq!(similar[0]["id"].as_i64().unwrap(), 1);
    assert_eq!(similar[0]["title"].as_str().unwrap(), "Upload Guidelines");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_wiki_articles"),
    migrations = "../storage/migrations"
)]
async fn test_unlink_similar_wiki_articles(pool: sqlx::PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::LinkSimilarWikiArticles,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/wiki/articles/similar")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"wiki_article_id_1": 1, "wiki_article_id_2": 2}))
        .to_request();
    assert_eq!(
        test::call_service(&service, req).await.status(),
        StatusCode::OK
    );

    let req = test::TestRequest::delete()
        .uri("/api/wiki/articles/similar")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"wiki_article_id_1": 2, "wiki_article_id_2": 1}))
        .to_request();
    assert_eq!(
        test::call_service(&service, req).await.status(),
        StatusCode::OK
    );

    let req = test::TestRequest::get()
        .uri("/api/wiki/articles?id=1")
        .insert_header(auth_header(&user.token))
        .to_request();
    let article: Value = call_and_read_body_json(&service, req).await;
    assert!(
        article["similar_wiki_articles"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_wiki_articles"),
    migrations = "../storage/migrations"
)]
async fn test_cannot_link_wiki_article_to_itself(pool: sqlx::PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::LinkSimilarWikiArticles,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/wiki/articles/similar")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"wiki_article_id_1": 1, "wiki_article_id_2": 1}))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
