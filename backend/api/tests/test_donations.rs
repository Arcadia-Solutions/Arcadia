use crate::{common, mocks};

use actix_web::{http::StatusCode, test};
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::donation::{Donation, SearchDonationsResponse};
use common::{
    TestUser, auth_header, call_and_read_body_json, call_and_read_body_json_with_status,
    create_test_app_and_login,
};
use mocks::mock_redis::MockRedisPool;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

// ============================================================================
// Search Donations Tests
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_search_donations_requires_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations?page=1&page_size=10")
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_search_donations_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SearchDonation).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations?page=1&page_size=10")
        .to_request();

    let response = call_and_read_body_json::<SearchDonationsResponse, _>(&service, req).await;

    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 10);
    assert_eq!(response.total_items, 6);
    assert_eq!(response.results.len(), 6);
    // Total amount: 50 + 75.5 + 100 + 25 + 30 + 150 = 430.5
    assert_eq!(response.total_amount, 430.5);
    // Unique donors: 100, 101, 102, 103 = 4
    assert_eq!(response.unique_donors_count, 4);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_search_donations_with_filters(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SearchDonation).await;

    // Filter by donated_by_id
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations?page=1&page_size=10&donated_by_id=100")
        .to_request();

    let response = call_and_read_body_json::<SearchDonationsResponse, _>(&service, req).await;

    assert_eq!(response.total_items, 2);
    assert_eq!(response.results.len(), 2);
    // Amount from user 100: 50 + 75.5 = 125.5
    assert_eq!(response.total_amount, 125.5);
    assert_eq!(response.unique_donors_count, 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_search_donations_with_amount_filter(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SearchDonation).await;

    // Filter by min_amount
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations?page=1&page_size=10&min_amount=100")
        .to_request();

    let response = call_and_read_body_json::<SearchDonationsResponse, _>(&service, req).await;

    assert_eq!(response.total_items, 2);
    // Donations >= 100: 100 + 150 = 250
    assert_eq!(response.total_amount, 250.0);
    assert_eq!(response.unique_donors_count, 2);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_search_donations_with_created_by_filter(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SearchDonation).await;

    // Filter by created_by_id
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations?page=1&page_size=10&created_by_id=122")
        .to_request();

    let response = call_and_read_body_json::<SearchDonationsResponse, _>(&service, req).await;

    assert_eq!(response.total_items, 5);
    // All donations created by user 122 except the one created by user 100
    assert_eq!(response.total_amount, 280.5);
}

// ============================================================================
// Create Donation Tests
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_create_donation_requires_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "donated_by_id": 100,
            "amount": 50.0,
            "note": "Test donation"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_create_donation_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateDonation).await;

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "donated_by_id": 100,
            "amount": 50.0,
            "note": "Test donation"
        }))
        .to_request();

    let donation =
        call_and_read_body_json_with_status::<Donation, _>(&service, req, StatusCode::CREATED)
            .await;

    assert_eq!(donation.donated_by_id, 100);
    assert_eq!(donation.amount, 50.0);
    assert_eq!(donation.note, Some("Test donation".to_string()));
    assert_eq!(donation.created_by_id, 122); // user_don_crt
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_create_donation_with_invalid_amount(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateDonation).await;

    // Test with zero amount
    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "donated_by_id": 100,
            "amount": 0.0,
            "note": "Invalid donation"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Test with negative amount
    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "donated_by_id": 100,
            "amount": -10.0,
            "note": "Invalid donation"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_create_donation_with_nonexistent_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateDonation).await;

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "donated_by_id": 999999,
            "amount": 50.0,
            "note": "Test donation"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Edit Donation Tests
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_edit_donation_as_creator_with_create_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateDonation).await;

    // User 122 (user_don_crt) created donation id 1
    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "id": 1,
            "donated_by_id": 100,
            "donated_at": "2024-01-15T10:00:00Z",
            "amount": 60.0,
            "note": "Updated donation"
        }))
        .to_request();

    let donation = call_and_read_body_json::<Donation, _>(&service, req).await;

    assert_eq!(donation.id, 1);
    assert_eq!(donation.amount, 60.0);
    assert_eq!(donation.note, Some("Updated donation".to_string()));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_edit_donation_as_non_creator_without_edit_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateDonation).await;

    // User 122 (user_don_crt) did NOT create donation id 6 (created by user 100)
    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "id": 6,
            "donated_by_id": 103,
            "donated_at": "2024-04-01T08:00:00Z",
            "amount": 200.0,
            "note": "Trying to edit someone else's donation"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_edit_donation_with_edit_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditDonation).await;

    // User 123 (user_don_edit) has edit_donation permission, can edit any donation
    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "id": 6,
            "donated_by_id": 103,
            "donated_at": "2024-04-01T08:00:00Z",
            "amount": 200.0,
            "note": "Edited by admin"
        }))
        .to_request();

    let donation = call_and_read_body_json::<Donation, _>(&service, req).await;

    assert_eq!(donation.id, 6);
    assert_eq!(donation.amount, 200.0);
    assert_eq!(donation.note, Some("Edited by admin".to_string()));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_edit_donation_with_invalid_amount(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditDonation).await;

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "id": 1,
            "donated_by_id": 100,
            "donated_at": "2024-01-15T10:00:00Z",
            "amount": 0.0,
            "note": "Invalid"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_edit_nonexistent_donation(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditDonation).await;

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "id": 999999,
            "donated_by_id": 100,
            "donated_at": "2024-01-15T10:00:00Z",
            "amount": 50.0,
            "note": "Does not exist"
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// Delete Donation Tests
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_delete_donation_requires_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "id": 1
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_delete_donation_with_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::DeleteDonation,
    )
    .await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "id": 1
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the donation was deleted by searching
    let (service2, user2) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::SearchDonation,
    )
    .await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user2.token))
        .uri("/api/donations?page=1&page_size=10")
        .to_request();

    let response = call_and_read_body_json::<SearchDonationsResponse, _>(&service2, req).await;

    assert_eq!(response.total_items, 5); // Down from 6
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_delete_nonexistent_donation(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::DeleteDonation).await;

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations")
        .set_json(json!({
            "id": 999999
        }))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// Aggregates and Complex Query Tests
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_search_donations_ordering(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SearchDonation).await;

    // Order by amount descending
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations?page=1&page_size=10&order_by_column=amount&order_by_direction=desc")
        .to_request();

    let response = call_and_read_body_json::<SearchDonationsResponse, _>(&service, req).await;

    assert_eq!(response.results.len(), 6);
    // First should be the highest amount (150.0)
    assert_eq!(response.results[0].amount, 150.0);
    // Last should be the lowest amount (25.0)
    assert_eq!(response.results[5].amount, 25.0);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_donations"),
    migrations = "../storage/migrations"
)]
async fn test_search_donations_pagination(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::SearchDonation).await;

    // Get first page with 2 items
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations?page=1&page_size=2")
        .to_request();

    let response = call_and_read_body_json::<SearchDonationsResponse, _>(&service, req).await;

    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 2);
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.total_items, 6);

    // Get second page
    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/donations?page=2&page_size=2")
        .to_request();

    let response = call_and_read_body_json::<SearchDonationsResponse, _>(&service, req).await;

    assert_eq!(response.page, 2);
    assert_eq!(response.results.len(), 2);
}
