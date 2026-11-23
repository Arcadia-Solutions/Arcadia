mod common;

use actix_web::{App, test};
use arcadia_shared::tracker::models::user::{APIInsertUser, Passkey};
use arcadia_tracker::routes::init;
use sqlx::PgPool;
use std::str::FromStr;

#[sqlx::test(fixtures("with_test_user"))]
async fn test_upsert_user_drops_the_previous_passkey(pool: PgPool) {
    let tracker = common::create_test_tracker(pool, common::test_config()).await;
    let service = test::init_service(App::new().app_data(tracker.clone()).configure(init)).await;

    let previous_passkey = Passkey::from_str("d2037c66dd3e13044e0d2f9b891c3837").unwrap();
    let new_passkey = Passkey::from_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab").unwrap();
    let user_id = *tracker.passkey2id.read().get(&previous_passkey).unwrap();

    let req = test::TestRequest::put()
        .uri("/api/users")
        .insert_header(("x-api-key", "amazing_api_key_of_32_characters"))
        .set_json(APIInsertUser {
            id: user_id,
            passkey: new_passkey,
            max_snatches_per_day: None,
        })
        .to_request();
    assert!(
        test::call_service(&service, req)
            .await
            .status()
            .is_success()
    );

    let passkey2id = tracker.passkey2id.read();
    assert_eq!(passkey2id.get(&previous_passkey), None);
    assert_eq!(passkey2id.get(&new_passkey), Some(&user_id));
}
