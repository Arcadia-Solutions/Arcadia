use crate::mocks;

use std::sync::Arc;

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::{
        title_group_stats::{
            CatalogKind, TitleGroupAttributeCountDataPoint, TitleGroupStatsResponse,
        },
        user::LoginResponse,
    },
};
use mocks::mock_redis::MockRedisPool;
use serde::Deserialize;
use sqlx::PgPool;

use crate::common::{
    TestUser, auth_header, call_and_read_body_json_with_status, create_test_app_and_login,
};

/// The stats of an artist, a series or a collage ride along with the rest of its information, so a
/// test only deserializes the field it asserts on.
#[derive(Debug, Deserialize)]
struct CatalogWithStats {
    title_group_stats: TitleGroupStatsResponse,
}

/// Aggregates `data_points` into a comparable map, so a test only asserts on the values it cares
/// about and not on their order.
fn counts_by_attribute(data_points: &[TitleGroupAttributeCountDataPoint]) -> Vec<(String, i64)> {
    let mut counts: Vec<(String, i64)> = data_points
        .iter()
        .map(|point| (point.attribute_value.clone(), point.count))
        .collect();
    counts.sort();
    counts
}

async fn get_stats(
    service: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    uri: &str,
    user: &LoginResponse,
) -> TitleGroupStatsResponse {
    let req = test::TestRequest::get()
        .uri(uri)
        .insert_header(auth_header(&user.token))
        .to_request();

    let catalog: CatalogWithStats =
        call_and_read_body_json_with_status(service, req, StatusCode::OK).await;

    catalog.title_group_stats
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_stats"),
    migrations = "../storage/migrations"
)]
async fn test_title_group_catalog_stats_of_artist(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    // any logged in user can read them, unlike the site wide stats
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let stats = get_stats(&service, "/api/artists?id=20", &user).await;

    // title groups 20, 21 and 22 are affiliated to the artist, 23 and 24 are not
    let mut release_years: Vec<(Option<i32>, i64)> = stats
        .title_groups_per_release_year
        .iter()
        .map(|point| (point.year, point.count))
        .collect();
    release_years.sort();
    assert_eq!(
        release_years,
        vec![(None, 1), (Some(2020), 1), (Some(2021), 1)]
    );

    assert_eq!(
        counts_by_attribute(&stats.content_types),
        vec![("movie".into(), 2), ("music".into(), 1)]
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_stats"),
    migrations = "../storage/migrations"
)]
async fn test_title_group_catalog_stats_of_series(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let stats = get_stats(&service, "/api/series?id=20", &user).await;

    // only title groups 20 and 21 belong to the series
    let release_years: Vec<(Option<i32>, i64)> = stats
        .title_groups_per_release_year
        .iter()
        .map(|point| (point.year, point.count))
        .collect();
    assert_eq!(release_years, vec![(Some(2020), 1), (Some(2021), 1)]);

    assert_eq!(
        counts_by_attribute(&stats.content_types),
        vec![("movie".into(), 1), ("music".into(), 1)]
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_stats"),
    migrations = "../storage/migrations"
)]
async fn test_title_group_catalog_stats_of_collage(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let stats = get_stats(&service, "/api/collages?id=20", &user).await;

    // only title groups 22 and 23 are entries of the collage
    let mut release_years: Vec<(Option<i32>, i64)> = stats
        .title_groups_per_release_year
        .iter()
        .map(|point| (point.year, point.count))
        .collect();
    release_years.sort();
    assert_eq!(release_years, vec![(None, 1), (Some(1999), 1)]);

    assert_eq!(
        counts_by_attribute(&stats.content_types),
        vec![("book".into(), 1), ("movie".into(), 1)]
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_stats"),
    migrations = "../storage/migrations"
)]
async fn test_title_group_catalog_stats_of_catalog_without_title_groups(pool: PgPool) {
    let pool = ConnectionPool::with_pg_pool(pool);

    let stats = pool
        .get_title_group_catalog_stats(CatalogKind::Artist, 999999)
        .await
        .unwrap();

    assert!(stats.title_groups_per_release_year.is_empty());
    assert!(stats.content_types.is_empty());
}
