pub mod common;
pub mod mocks;

use std::io::{Cursor, Read};
use std::sync::Arc;

use actix_web::{http::StatusCode, test};
use arcadia_storage::connection_pool::ConnectionPool;
use mocks::mock_redis::MockRedisPool;
use serde::Deserialize;
use sqlx::PgPool;

use crate::common::{auth_header, TestUser};

const BASIC_USER_PASSKEY: &str = "d2037c66dd3e13044e0d2f9b891c3837";

/// Opens the zip archive returned by the endpoint and returns the name and bytes of every entry.
fn read_archive_entries(body: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(body.to_vec())).expect("response body is a valid zip");

    (0..archive.len())
        .map(|index| {
            let mut entry = archive.by_index(index).expect("zip entry can be read");
            let name = entry.name().expect("zip entry name").to_string();
            let mut contents = Vec::new();
            entry.read_to_end(&mut contents).expect("zip entry content");
            (name, contents)
        })
        .collect()
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_user_torrents_archive"
    ),
    migrations = "../storage/migrations"
)]
async fn test_download_uploaded_torrents_archive(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/me/torrents-archive?type=uploaded")
        .to_request();

    let resp = test::call_service(&service, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(actix_web::http::header::CONTENT_TYPE)
            .map(|value| value.to_str().unwrap().to_string()),
        Some("application/zip".to_string())
    );

    let body = test::read_body(resp).await;
    let entries = read_archive_entries(&body);

    // The basic user uploaded exactly one torrent with a valid info_dict (torrent 1).
    let torrent_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.0.ends_with(".torrent"))
        .collect();
    assert_eq!(
        torrent_entries.len(),
        1,
        "expected one torrent in the archive"
    );
    assert!(
        torrent_entries[0].0.ends_with(".torrent"),
        "entry should be a .torrent file, got {}",
        torrent_entries[0].0
    );
    // Entries are prefixed with the site name, like single downloads in the frontend.
    assert_eq!(
        torrent_entries[0].0,
        "[Arcadia] The Beatles - Love Me Do - P.S. I Love You (Parlophone Single) [24-96] (1).torrent"
    );

    #[derive(Debug, Deserialize)]
    struct MetaInfo {
        announce: String,
    }

    let metainfo: MetaInfo = serde_bencode::from_bytes(&torrent_entries[0].1)
        .expect("archived file is a valid .torrent");
    assert!(
        metainfo.announce.contains(BASIC_USER_PASSKEY),
        "announce url should contain the downloader's passkey"
    );
    // Export metadata is always appended
    assert!(
        entries.iter().any(|e| e.0 == "export-metadata.txt"),
        "expected export-metadata.txt in the archive"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_user_torrents_archive"
    ),
    migrations = "../storage/migrations"
)]
async fn test_download_snatched_torrents_archive(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/me/torrents-archive?type=snatched")
        .to_request();

    let resp = test::call_service(&service, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let entries = read_archive_entries(&body);

    // The basic user snatched exactly one torrent (torrent 1).
    let torrent_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.0.ends_with(".torrent"))
        .collect();
    assert_eq!(
        torrent_entries.len(),
        1,
        "expected one snatched torrent in the archive"
    );
    assert!(torrent_entries[0].0.ends_with(".torrent"));
    assert!(
        entries.iter().any(|e| e.0 == "export-metadata.txt"),
        "expected export-metadata.txt in the archive"
    );
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_download_torrents_archive_rejects_unknown_type(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        common::create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/me/torrents-archive?type=everything")
        .to_request();

    let resp = test::call_service(&service, req).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
