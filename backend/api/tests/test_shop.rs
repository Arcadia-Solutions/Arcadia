use crate::{common, mocks};

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::shop::{BuyFreeleechTokensRequest, BuyUploadRequest, ShopPricing};
use common::{TestUser, auth_header, call_and_read_body_json, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

const BYTES_PER_GB: i64 = 1_073_741_824;

#[sqlx::test(
    fixtures("with_test_users", "with_test_shop_balance"),
    migrations = "../storage/migrations"
)]
async fn test_get_shop_pricing_no_promotion(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/shop/pricing")
        .insert_header(auth_header(&user.token))
        .to_request();

    let pricing: ShopPricing = call_and_read_body_json(&service, req).await;

    assert_eq!(pricing.upload_base_price_per_gb, Some(100));
    assert_eq!(pricing.freeleech_token_base_price, Some(500));
    assert!(!pricing.upload_discount_tiers.as_ref().unwrap().is_empty());
    assert!(
        !pricing
            .freeleech_token_discount_tiers
            .as_ref()
            .unwrap()
            .is_empty()
    );
    // No next class available for newbie in default config
    assert!(pricing.promotion.is_none());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_shop_balance", "with_buyable_promotion"),
    migrations = "../storage/migrations"
)]
async fn test_get_shop_pricing_with_promotion(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/shop/pricing")
        .insert_header(auth_header(&user.token))
        .to_request();

    let pricing: ShopPricing = call_and_read_body_json(&service, req).await;

    let promotion = pricing.promotion.expect("promotion should be available");
    assert_eq!(promotion.next_class_name, "member");
    assert_eq!(promotion.cost, 5000);
    // User has 10000 BP so requirements_met depends on class requirements (none set)
    assert!(promotion.requirements_met);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_shop_balance"),
    migrations = "../storage/migrations"
)]
async fn test_buy_upload_success(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let user_before = pool.find_user_with_id(100).await.unwrap();
    assert_eq!(user_before.bonus_points, 10000);
    assert_eq!(user_before.uploaded, 0);

    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let request = BuyUploadRequest {
        bytes: 5 * BYTES_PER_GB,
    };

    let req = test::TestRequest::post()
        .uri("/api/shop/buy-upload")
        .insert_header(auth_header(&user.token))
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let user_after = pool.find_user_with_id(100).await.unwrap();
    assert_eq!(user_after.bonus_points, 9500);
    assert_eq!(user_after.uploaded, 5 * BYTES_PER_GB);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_shop_balance"),
    migrations = "../storage/migrations"
)]
async fn test_buy_upload_with_discount(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    // Buy 10GB to get 10% discount (threshold is 10GB)
    let request = BuyUploadRequest {
        bytes: 10 * BYTES_PER_GB,
    };

    let req = test::TestRequest::post()
        .uri("/api/shop/buy-upload")
        .insert_header(auth_header(&user.token))
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Base: 10GB * 100 = 1000, with 10% discount = 900
    let user_after = pool.find_user_with_id(100).await.unwrap();
    assert_eq!(user_after.bonus_points, 10000 - 900);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_shop_balance"),
    migrations = "../storage/migrations"
)]
async fn test_buy_upload_insufficient_points(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    // Try to buy 200GB (costs 14000 BP with 30% discount, user has 10000)
    let request = BuyUploadRequest {
        bytes: 200 * BYTES_PER_GB,
    };

    let req = test::TestRequest::post()
        .uri("/api/shop/buy-upload")
        .insert_header(auth_header(&user.token))
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_shop_balance"),
    migrations = "../storage/migrations"
)]
async fn test_buy_freeleech_tokens_success(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let user_before = pool.find_user_with_id(100).await.unwrap();
    assert_eq!(user_before.freeleech_tokens, 0);

    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool),
        MockRedisPool::default(),
        TestUser::Standard,
    )
    .await;

    let request = BuyFreeleechTokensRequest { quantity: 3 };

    let req = test::TestRequest::post()
        .uri("/api/shop/buy-freeleech-tokens")
        .insert_header(auth_header(&user.token))
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let user_after = pool.find_user_with_id(100).await.unwrap();
    assert_eq!(user_after.freeleech_tokens, 3);
    assert_eq!(user_after.bonus_points, 8500);
}
