use crate::{common, mocks};

use actix_web::test;
use arcadia_api::config::{Config, OidcConfig};
use arcadia_storage::connection_pool::ConnectionPool;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use common::{
    TestUser, auth_header, call_and_read_body_json, create_test_app_with_config, login_as,
    spawn_http_server,
};
use mocks::mock_redis::MockRedisPool;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

const ELYSIA_IMAGE_URL: &str = "https://elysia.example.com/i/abcdefgh.png";

#[derive(Deserialize)]
struct UploadImageResponse {
    url: String,
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_uploading_an_image_to_elysia_keeps_the_uploader(pool: PgPool) {
    let (elysia_url, received_request) = spawn_http_server(
        "200 OK",
        serde_json::json!({ "url": ELYSIA_IMAGE_URL }).to_string(),
    );
    let mut config = arcadia_shared::config::load::<Config>();
    config.image_host.elysia_url = Some(elysia_url);
    config.oidc = Some(OidcConfig {
        issuer: "http://localhost:8080/api/oauth".parse().unwrap(),
        client_id: String::from("elysia"),
        client_secret: String::from("elysia-secret"),
        redirect_uri: String::from("http://localhost:3000/auth/sso/callback"),
    });
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, _) = create_test_app_with_config(pool, MockRedisPool::default(), config).await;
    let user = login_as(&service, TestUser::Standard).await;

    let boundary = "testboundary";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"image\"; filename=\"x.png\"\r\nContent-Type: image/png\r\n\r\nimage-bytes\r\n--{boundary}--\r\n"
    );
    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .insert_header((
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        ))
        .uri("/api/image-host/upload")
        .set_payload(body)
        .to_request();

    let response = call_and_read_body_json::<UploadImageResponse, _>(&service, req).await;
    assert_eq!(response.url, ELYSIA_IMAGE_URL);

    let request = received_request.recv().unwrap().to_lowercase();
    assert!(request.starts_with("post /api/upload "));
    assert!(request.contains(&format!(
        "authorization: basic {}",
        BASE64.encode("elysia:elysia-secret").to_lowercase()
    )));
    assert!(request.contains("on-behalf-of-sub: 100\r\n"));
    assert!(request.contains("on-behalf-of-username: user_basic\r\n"));
    assert!(request.contains("image-bytes"));
    assert!(request.contains("filename=\"x.png\""));
}
