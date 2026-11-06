pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::emoji::{
    Emoji, EmojiEnabledUpdate, EmojiUsage, ReorderEmojiEntry, ReorderEmojis,
};
use common::{TestUser, auth_header, create_test_app_and_login, login_as};
use mocks::mock_redis::MockRedisPool;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
}

fn multipart_body(fields: &[(&str, &str)], image: Option<(&str, &[u8])>) -> (Vec<u8>, String) {
    let boundary = "arcadiatestboundary";
    let mut body: Vec<u8> = Vec::new();
    for (name, value) in fields {
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
            )
            .as_bytes(),
        );
    }
    if let Some((mime_type, bytes)) = image {
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"emoji\"\r\nContent-Type: {mime_type}\r\n\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (body, format!("multipart/form-data; boundary={boundary}"))
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_creates_unicode_and_image_emojis(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let (body, content_type) =
        multipart_body(&[("name", "party"), ("unicode_character", "🎉")], None);
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let party_emoji: Emoji =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    assert_eq!(party_emoji.unicode_character.as_deref(), Some("🎉"));

    let (body, content_type) = multipart_body(
        &[("name", "custom")],
        Some(("image/png", &[0x89, 0x50, 0x4e, 0x47])),
    );
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let custom_emoji: Emoji =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    assert!(custom_emoji.unicode_character.is_none());

    // The server assigns the sort order itself: the second emoji lands right after the first.
    assert_eq!(party_emoji.sort_order + 1, custom_emoji.sort_order);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_emoji_payload_validation(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    // Neither a character nor an image.
    let (body, content_type) = multipart_body(&[("name", "empty")], None);
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;
    assert_eq!(
        error.error,
        "an emoji must have either a unicode character or an image, and not both"
    );

    // Both a character and an image.
    let (body, content_type) = multipart_body(
        &[("name", "both"), ("unicode_character", "🎉")],
        Some(("image/png", &[0x89, 0x50, 0x4e, 0x47])),
    );
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;
    assert_eq!(
        error.error,
        "an emoji must have either a unicode character or an image, and not both"
    );

    // Wrong mime type.
    let (body, content_type) =
        multipart_body(&[("name", "bad_mime")], Some(("image/bmp", &[0x42, 0x4d])));
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;
    assert_eq!(
        error.error,
        "the emoji image must be a png, a webp, a gif or an svg"
    );

    // Image over 32 KB.
    let oversized = vec![0x89u8; 32769];
    let (body, content_type) =
        multipart_body(&[("name", "too_big")], Some(("image/png", &oversized)));
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;
    assert_eq!(error.error, "the emoji image is larger than 32 KiB");

    // A blank unicode character counts as no representation at all, rather than an emoji that
    // renders as nothing.
    let (body, content_type) = multipart_body(
        &[("name", "blank_character"), ("unicode_character", "   ")],
        None,
    );
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;
    assert_eq!(
        error.error,
        "an emoji must have either a unicode character or an image, and not both"
    );

    // A blank name
    let name = "  ";
    let (body, content_type) = multipart_body(&[("name", name), ("unicode_character", "🎉")], None);
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;
    assert_eq!(error.error, "the emoji name must not be blank");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_editing_with_a_blank_unicode_character_keeps_the_image(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    // The form always submits its unicode character field, empty when the emoji is an image one:
    // that must be read as "unchanged", not as "drop the image".
    let (body, content_type) = multipart_body(
        &[
            ("id", "102"),
            ("name", "  custom_smile_renamed  "),
            ("unicode_character", ""),
        ],
        None,
    );
    let req = test::TestRequest::put()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let edited: Emoji =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert!(edited.unicode_character.is_none());
    // The name is stored trimmed, so it cannot collide only by surrounding whitespace.
    assert_eq!(edited.name, "custom_smile_renamed");

    let req = test::TestRequest::get()
        .uri("/api/emojis/102/image")
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_duplicate_emoji_name_is_rejected(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    // Creating an emoji named after an existing one must fail with a clear 400, not a generic
    // database error.
    let (body, content_type) =
        multipart_body(&[("name", "thumbs_up"), ("unicode_character", "🙂")], None);
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;
    assert_eq!(error.error, "an emoji with this name already exists");

    // Renaming an emoji to a name already taken by another emoji must fail the same way.
    let (body, content_type) = multipart_body(
        &[
            ("id", "101"),
            ("name", "thumbs_up"),
            ("unicode_character", "👎"),
        ],
        None,
    );
    let req = test::TestRequest::put()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;
    assert_eq!(error.error, "an emoji with this name already exists");
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis",
        "with_test_forum_post_reactions"
    ),
    migrations = "../storage/migrations"
)]
async fn test_list_edit_and_delete_emojis(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let emojis: Vec<Emoji> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(emojis.len(), 3);
    assert_eq!(emojis[0].name, "thumbs_up");
    let version_before_edit = emojis[2].image_version;

    let (body, content_type) = multipart_body(
        &[("id", "102"), ("name", "custom_smile")],
        Some(("image/png", &[0x89, 0x50, 0x4e, 0x47, 0x0d])),
    );
    let req = test::TestRequest::put()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let edited: Emoji =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    // `image_version` is `EXTRACT(EPOCH FROM updated_at)::BIGINT`, whole seconds only, and this
    // test consistently runs in well under a second, so a strict `>` here is not a stronger
    // assertion, it is a guaranteed failure: see the report for why item 11 keeps `>=`.
    assert!(edited.image_version >= version_before_edit);
    // Editing no longer takes a sort order, it must stay whatever the fixture set it to.
    assert_eq!(edited.sort_order, 3);

    // The fixture gives emoji 100 two reactions on forum post 100: deleting the emoji must
    // cascade and remove them, leaving only the reaction on emoji 101.
    let req = test::TestRequest::delete()
        .uri("/api/emojis?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let reactions = pool
        .find_reactions_for_forum_posts(&[100], 100)
        .await
        .unwrap();
    let post_reactions = reactions.get(&100).expect("emoji 101's reaction remains");
    assert_eq!(post_reactions.len(), 1);
    assert_eq!(post_reactions[0].emoji_id, 101);

    let req = test::TestRequest::delete()
        .uri("/api/emojis?id=100")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_regular_user_cannot_manage_emojis(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // Reading the catalogue is allowed for everyone.
    let req = test::TestRequest::get()
        .uri("/api/emojis")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let (body, content_type) =
        multipart_body(&[("name", "party"), ("unicode_character", "🎉")], None);
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&user.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let (body, content_type) = multipart_body(
        &[
            ("id", "100"),
            ("name", "thumbs_up"),
            ("unicode_character", "👍"),
        ],
        None,
    );
    let req = test::TestRequest::put()
        .uri("/api/emojis")
        .insert_header(auth_header(&user.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let req = test::TestRequest::delete()
        .uri("/api/emojis?id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis",
        "with_test_forum_post_reactions"
    ),
    migrations = "../storage/migrations"
)]
async fn test_emojis_usage(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/emojis/usage")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let usage: Vec<EmojiUsage> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // The fixture gives emoji 100 two reactions, emoji 101 one reaction, and emoji 102 none.
    let reactions_of = |emoji_id: i32| {
        usage
            .iter()
            .find(|entry| entry.emoji_id == emoji_id)
            .map(|entry| entry.reactions_amount)
    };
    assert_eq!(reactions_of(100), Some(2));
    assert_eq!(reactions_of(101), Some(1));
    assert_eq!(reactions_of(102), Some(0));

    let user = login_as(&service, TestUser::Standard).await;
    let req = test::TestRequest::get()
        .uri("/api/emojis/usage")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_edit_emoji_keeps_current_representation_when_unspecified(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    // Emoji 100 is the unicode "thumbs_up" emoji. Renaming it without sending a character or
    // an image must keep its current unicode character.
    let (body, content_type) =
        multipart_body(&[("id", "100"), ("name", "thumbs_up_renamed")], None);
    let req = test::TestRequest::put()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let edited: Emoji =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(edited.name, "thumbs_up_renamed");
    assert_eq!(edited.unicode_character.as_deref(), Some("👍"));
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_edit_emoji_switching_to_an_image_clears_the_unicode_character(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    // Emoji 100 is the unicode "thumbs_up" emoji. Switching it to an image must clear its
    // unicode character rather than keeping both representations.
    let (body, content_type) = multipart_body(
        &[("id", "100"), ("name", "thumbs_up")],
        Some(("image/png", &[0x89, 0x50, 0x4e, 0x47])),
    );
    let req = test::TestRequest::put()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let edited: Emoji =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert!(edited.unicode_character.is_none());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_emoji_image_is_served_without_authentication(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = common::create_test_app(pool, MockRedisPool::default()).await;

    let req = test::TestRequest::get()
        .uri("/api/emojis/102/image")
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("image/png")
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("public, max-age=31536000, immutable")
    );
    let etag = response
        .headers()
        .get("etag")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    assert!(etag.is_some(), "the image response must carry an ETag");
    // An emoji image can be an SVG carrying script, so every emoji image response, not only
    // SVG ones, must be hardened against being treated as an active document.
    assert_eq!(
        response
            .headers()
            .get("content-security-policy")
            .and_then(|value| value.to_str().ok()),
        Some("default-src 'none'; style-src 'unsafe-inline'; sandbox")
    );
    assert_eq!(
        response
            .headers()
            .get("x-content-type-options")
            .and_then(|value| value.to_str().ok()),
        Some("nosniff")
    );

    // The ETag is stable across requests, since it is derived from the emoji's `updated_at`.
    let req = test::TestRequest::get()
        .uri("/api/emojis/102/image")
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(
        response
            .headers()
            .get("etag")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string),
        etag
    );

    // A unicode emoji has no image.
    let req = test::TestRequest::get()
        .uri("/api/emojis/100/image")
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_emoji_management_requires_authentication(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = common::create_test_app(pool, MockRedisPool::default()).await;

    // Unlike the image route, the rest of the emoji route group requires authentication, even
    // for routes that are only read-only or fail on missing fields.
    let req = test::TestRequest::get().uri("/api/emojis").to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let req = test::TestRequest::get()
        .uri("/api/emojis/usage")
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let (body, content_type) = multipart_body(&[], None);
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_creates_and_serves_an_svg_emoji(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
    let (body, content_type) =
        multipart_body(&[("name", "vector_smile")], Some(("image/svg+xml", svg)));
    let req = test::TestRequest::post()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .insert_header(("content-type", content_type))
        .set_payload(body)
        .to_request();
    let emoji: Emoji =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;
    assert!(emoji.unicode_character.is_none());

    let req = test::TestRequest::get()
        .uri(&format!("/api/emojis/{}/image", emoji.id))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("image/svg+xml")
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_staff_reorders_emojis(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    // The fixture orders emojis 100 (thumbs_up), 101 (thumbs_down), 102 (custom_smile).
    // Swapping the sort orders of 100 and 101 must move thumbs_down ahead of thumbs_up.
    let reorder_body = ReorderEmojis {
        emojis: vec![
            ReorderEmojiEntry {
                id: 101,
                sort_order: 1,
            },
            ReorderEmojiEntry {
                id: 100,
                sort_order: 2,
            },
        ],
    };
    let req = test::TestRequest::put()
        .uri("/api/emojis/reorder")
        .insert_header(auth_header(&staff.token))
        .set_json(&reorder_body)
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let req = test::TestRequest::get()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let emojis: Vec<Emoji> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let names: Vec<&str> = emojis.iter().map(|emoji| emoji.name.as_str()).collect();
    assert_eq!(names, vec!["thumbs_down", "thumbs_up", "custom_smile"]);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_reordering_an_unknown_emoji_is_not_found(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let reorder_body = ReorderEmojis {
        emojis: vec![ReorderEmojiEntry {
            id: 999,
            sort_order: 1,
        }],
    };
    let req = test::TestRequest::put()
        .uri("/api/emojis/reorder")
        .insert_header(auth_header(&staff.token))
        .set_json(&reorder_body)
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_reordering_a_mix_of_known_and_unknown_emojis_changes_nothing(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    // Emoji 100 is a valid target, 999 does not exist. The whole request must be rejected, and
    // emoji 100 must keep its original sort order rather than the update for 100 being applied
    // before the unknown id 999 is discovered.
    let reorder_body = ReorderEmojis {
        emojis: vec![
            ReorderEmojiEntry {
                id: 100,
                sort_order: 5,
            },
            ReorderEmojiEntry {
                id: 999,
                sort_order: 6,
            },
        ],
    };
    let req = test::TestRequest::put()
        .uri("/api/emojis/reorder")
        .insert_header(auth_header(&staff.token))
        .set_json(&reorder_body)
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let req = test::TestRequest::get()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let emojis: Vec<Emoji> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let emoji_100 = emojis
        .iter()
        .find(|emoji| emoji.id == 100)
        .expect("emoji 100 still exists");
    assert_eq!(
        emoji_100.sort_order, 1,
        "the partial update must have been rolled back"
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_regular_user_cannot_reorder_emojis(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let reorder_body = ReorderEmojis {
        emojis: vec![
            ReorderEmojiEntry {
                id: 101,
                sort_order: 1,
            },
            ReorderEmojiEntry {
                id: 100,
                sort_order: 2,
            },
        ],
    };
    let req = test::TestRequest::put()
        .uri("/api/emojis/reorder")
        .insert_header(auth_header(&user.token))
        .set_json(&reorder_body)
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_staff_toggles_an_emoji_disabled_and_back_enabled(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/emojis/enabled")
        .insert_header(auth_header(&staff.token))
        .set_json(&EmojiEnabledUpdate {
            id: 100,
            enabled: false,
        })
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let req = test::TestRequest::get()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let emojis: Vec<Emoji> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    // A disabled emoji is still listed, it just carries `enabled: false`, since the staff
    // screen needs the full catalogue and only the picker filters it out.
    assert_eq!(emojis.len(), 3);
    let emoji_100 = emojis.iter().find(|emoji| emoji.id == 100).unwrap();
    assert!(!emoji_100.enabled);

    let req = test::TestRequest::put()
        .uri("/api/emojis/enabled")
        .insert_header(auth_header(&staff.token))
        .set_json(&EmojiEnabledUpdate {
            id: 100,
            enabled: true,
        })
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let req = test::TestRequest::get()
        .uri("/api/emojis")
        .insert_header(auth_header(&staff.token))
        .to_request();
    let emojis: Vec<Emoji> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let emoji_100 = emojis.iter().find(|emoji| emoji.id == 100).unwrap();
    assert!(emoji_100.enabled);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_toggling_an_unknown_emoji_enabled_state_is_not_found(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/emojis/enabled")
        .insert_header(auth_header(&staff.token))
        .set_json(&EmojiEnabledUpdate {
            id: 999,
            enabled: false,
        })
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_emojis"),
    migrations = "../storage/migrations"
)]
async fn test_regular_user_cannot_toggle_emoji_enabled(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::put()
        .uri("/api/emojis/enabled")
        .insert_header(auth_header(&user.token))
        .set_json(&EmojiEnabledUpdate {
            id: 100,
            enabled: false,
        })
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
