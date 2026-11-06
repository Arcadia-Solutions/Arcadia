use crate::{common, mocks};

use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::artist::ArtistSearchResult;
use arcadia_storage::models::common::PaginatedResults;
use common::{TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

const MAX_PAGE_SIZE: u32 = 100;

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_page_size_bigger_than_the_maximum_is_capped(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri(
            "/api/search/artists?page=1&page_size=9999&order_by_column=name&order_by_direction=asc",
        )
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<ArtistSearchResult> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.page_size, MAX_PAGE_SIZE);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_page_size_smaller_than_the_maximum_is_kept(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/search/artists?page=1&page_size=20&order_by_column=name&order_by_direction=asc")
        .insert_header(auth_header(&user.token))
        .to_request();

    let results: PaginatedResults<ArtistSearchResult> =
        call_and_read_body_json(&service, req).await;

    assert_eq!(results.page_size, 20);
}
