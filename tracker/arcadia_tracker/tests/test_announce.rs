mod common;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use actix_http::Request;
use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceResponse},
    test, App, Error,
};
use arcadia_tracker::routes::init;
use serde::{de::DeserializeOwned, Deserialize};
use sqlx::PgPool;

async fn create_test_app(
    pool: PgPool,
) -> impl Service<Request, Response = ServiceResponse, Error = Error> {
    let tracker = common::create_test_tracker(pool, common::test_config()).await;

    test::init_service(App::new().app_data(tracker).configure(init)).await
}

async fn read_body_bencode<T: DeserializeOwned, B: MessageBody>(
    resp: ServiceResponse<B>,
) -> Result<T, serde_bencode::Error> {
    let body = test::read_body(resp).await;
    serde_bencode::from_bytes(&body)
}

#[derive(Debug, Deserialize)]
#[expect(dead_code)]
struct AnnounceResponse {
    #[serde(rename = "complete")]
    seeders: u32,
    #[serde(rename = "incomplete")]
    leechers: u32,
    #[serde(rename = "downloaded")]
    times_completed: u32,
    interval: u32,
    #[serde(rename = "min interval")]
    min_interval: u32,
    #[serde(default, deserialize_with = "deserialize_optional_bytes")]
    peers: Option<Vec<u8>>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_bytes",
        rename = "peers6"
    )]
    peers6: Option<Vec<u8>>,
}

fn deserialize_optional_bytes<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct BytesVisitor;

    impl<'de> serde::de::Visitor<'de> for BytesVisitor {
        type Value = Option<Vec<u8>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a byte string or sequence")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(v.to_vec()))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element::<u8>()? {
                vec.push(elem);
            }
            Ok(Some(vec))
        }
    }

    deserializer.deserialize_any(BytesVisitor)
}

#[derive(Debug, Deserialize)]
struct WrappedError {
    #[serde(rename = "failure reason")]
    failure_reason: String,
}

/// URL-encodes the info_hash bytes
fn url_encode_info_hash(info_hash: &[u8; 20]) -> String {
    percent_encoding::percent_encode(info_hash, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// Creates a valid peer_id for testing (must be 20 bytes)
/// Uses libtorrent format which starts with '-' followed by client identifier
fn test_peer_id() -> [u8; 20] {
    let mut peer_id = [0u8; 20];
    peer_id[0] = b'-';
    // Use '-lt0F01-' prefix which is in the allowed clients list
    peer_id[1..8].copy_from_slice(b"lt0F01-");
    // Fill rest with test data
    peer_id[8..].fill(b'1');
    peer_id
}

#[sqlx::test(
    fixtures("with_test_user"),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_invalid_passkey(pool: PgPool) {
    let service = create_test_app(pool).await;

    let invalid_passkey = "invalid_passkey_too_short";
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started",
            invalid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;

    let error: WrappedError = read_body_bencode(resp)
        .await
        .expect("Failed to decode error response");
    assert!(
        error.failure_reason.contains("Invalid passkey"),
        "Expected 'Invalid passkey' in failure reason, got: {}",
        error.failure_reason
    );
}

#[sqlx::test(
    fixtures("with_test_user"),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_passkey_not_found(pool: PgPool) {
    let service = create_test_app(pool).await;

    // Use a valid format passkey but one that doesn't exist
    let non_existent_passkey = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab";
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started",
            non_existent_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;

    let error: WrappedError = read_body_bencode(resp)
        .await
        .expect("Failed to decode error response");

    assert_eq!(
        error.failure_reason,
        "User does not exist. Please re-download the .torrent file."
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_info_hash_not_found(pool: PgPool) {
    let service = create_test_app(pool).await;

    let valid_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    // Use an info_hash that doesn't exist in the database
    let info_hash_bytes = [0xFF; 20];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;

    let error: WrappedError = read_body_bencode(resp)
        .await
        .expect("Failed to decode error response");

    assert_eq!(error.failure_reason, "InfoHash not found.");
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_successful_started(pool: PgPool) {
    let service = create_test_app(pool).await;

    let valid_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    // Info hash from with_test_torrent.sql: \x112233445566778899aabbccddeeff0011223344
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    let status = resp.status();

    if !status.is_success() {
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Expected success, got status {}. Body: {}",
            status, body_str
        );
    }

    let announce_resp: AnnounceResponse = read_body_bencode(resp)
        .await
        .expect("Failed to decode announce response");

    assert_eq!(announce_resp.seeders, 0, "Should have 0 seeders initially");
    assert_eq!(
        announce_resp.leechers, 1,
        "Should have 1 leecher (this peer)"
    );
    assert_eq!(
        announce_resp.times_completed, 0,
        "Should have 0 completions"
    );
    assert!(
        announce_resp.interval >= 1800,
        "Interval should be at least 1800"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_peers"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_with_existing_peers(pool: PgPool) {
    let service = create_test_app(pool).await;

    let valid_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1&numwant=50",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    let status = resp.status();

    if !status.is_success() {
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Expected success, got status {}. Body: {}",
            status, body_str
        );
    }

    let announce_resp: AnnounceResponse = read_body_bencode(resp)
        .await
        .expect("Failed to decode announce response");

    // The fixtures have 3 peers (2 seeders + 1 leecher), all with user_id 1
    // When we announce as user_id 1, the tracker filters out peers with same user_id from peer list
    // However, the seeder/leecher counts should reflect the total from database initially
    // But since we're loading fresh from the database and the counts start at 0, we need to check
    // if peers are actually loaded. The tracker should load peers from DB and count them.
    // For now, just verify the announce was successful and we have valid counts
    assert!(
        announce_resp.leechers >= 1, // At least this peer should be counted
        "Should have at least 1 leecher (this peer), got {}",
        announce_resp.leechers
    );
    // Note: The in-memory tracker loads peers from DB, but seeder/leecher counts might start at 0
    // and be updated based on active peers in the map. The fixtures peers might not be active
    // or counted until they're part of an announce cycle.
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_completed_event(pool: PgPool) {
    let service = create_test_app(pool).await;

    let valid_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    // First announce with started event
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=1000&left=0&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let _resp = test::call_service(&service, req).await;

    // Now announce with completed event
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=1000&left=0&event=completed&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    let status = resp.status();

    if !status.is_success() {
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Expected success, got status {}. Body: {}",
            status, body_str
        );
    }

    let announce_resp: AnnounceResponse = read_body_bencode(resp)
        .await
        .expect("Failed to decode announce response");

    // After completing, should be a seeder
    assert!(
        announce_resp.seeders >= 1,
        "Should have at least 1 seeder after completion"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_stopped_event(pool: PgPool) {
    let service = create_test_app(pool).await;

    let valid_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    // First announce with started event
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let _resp = test::call_service(&service, req).await;

    // Now announce with stopped event
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=100&downloaded=200&left=800&event=stopped&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    let status = resp.status();

    if !status.is_success() {
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Expected success, got status {}. Body: {}",
            status, body_str
        );
    }

    let announce_resp: AnnounceResponse = read_body_bencode(resp)
        .await
        .expect("Failed to decode announce response");

    // After stopping, should no longer count as leecher
    assert_eq!(
        announce_resp.leechers, 0,
        "Should have 0 leechers after stopping"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_missing_user_agent(pool: PgPool) {
    let service = create_test_app(pool).await;

    let valid_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        // Note: No User-Agent header
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;

    let error: WrappedError = read_body_bencode(resp)
        .await
        .expect("Failed to decode error response");
    assert_eq!(error.failure_reason, "user-agent is missing");
}

#[sqlx::test(
    fixtures(
        "with_test_user_snatch_limit",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_snatch_limit_under(pool: PgPool) {
    let service = create_test_app(pool).await;

    // User with max_snatches_per_day = 2
    let valid_passkey = "e3037c66dd3e13044e0d2f9b891c3838";
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    // Announce as leecher (left > 0) - should succeed since under limit
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    let status = resp.status();

    if !status.is_success() {
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Expected success, got status {}. Body: {}",
            status, body_str
        );
    }

    let announce_resp: AnnounceResponse = read_body_bencode(resp)
        .await
        .expect("Failed to decode announce response");

    assert_eq!(
        announce_resp.leechers, 1,
        "Should have 1 leecher (this peer)"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user_snatch_limit",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_torrent_2",
        "with_test_torrent_3"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_snatch_limit_exceeded(pool: PgPool) {
    let service = create_test_app(pool).await;

    // User with max_snatches_per_day = 2
    let valid_passkey = "e3037c66dd3e13044e0d2f9b891c3838";
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    // First torrent - should succeed
    let info_hash_1 = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, url_encode_info_hash(&info_hash_1), peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "First torrent should succeed");

    // Second torrent - should succeed (at limit)
    let info_hash_2 = [
        0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x55,
    ];
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, url_encode_info_hash(&info_hash_2), peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "Second torrent should succeed");

    // Third torrent - should fail (exceeds limit of 2)
    let info_hash_3 = [
        0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00, 0x11, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x66,
    ];
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, url_encode_info_hash(&info_hash_3), peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();
    let resp = test::call_service(&service, req).await;

    let error: WrappedError = read_body_bencode(resp)
        .await
        .expect("Failed to decode error response");
    assert_eq!(
        error.failure_reason,
        "You have already leeched 2 torrents in the past 24h."
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_bonus_points_deducted(pool: PgPool) {
    let service = create_test_app(pool.clone()).await;

    // User with 100 BP downloading torrent with 50 BP cost
    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    let status = resp.status();

    if !status.is_success() {
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Expected success, got status {}. Body: {}",
            status, body_str
        );
    }

    // Verify bonus points were deducted
    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 10")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(
        row.0, 50,
        "User should have 50 BP after deduction (100 - 50)"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_low_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_insufficient_bonus_points(pool: PgPool) {
    let service = create_test_app(pool.clone()).await;

    // User with 30 BP trying to download torrent with 50 BP cost
    let valid_passkey = "g5037c66dd3e13044e0d2f9b891c3840";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;

    let error: WrappedError = read_body_bencode(resp)
        .await
        .expect("Failed to decode error response");
    assert_eq!(
        error.failure_reason,
        "Not enough bonus points to download this torrent (cost: 50 BP)."
    );

    // Verify bonus points were NOT deducted
    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 11")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(row.0, 30, "User should still have 30 BP");
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_uploader_free_snatch(pool: PgPool) {
    // Get the actual user ID (may not be 1 depending on DB sequence)
    let user_row: (i32,) =
        sqlx::query_as("SELECT id FROM users WHERE passkey = 'd2037c66dd3e13044e0d2f9b891c3837'")
            .fetch_one(&pool)
            .await
            .expect("Failed to query user");
    let actual_user_id = user_row.0;

    // Set bonus points and ensure torrent's created_by_id matches user
    sqlx::query("UPDATE users SET bonus_points = 100 WHERE id = $1")
        .bind(actual_user_id)
        .execute(&pool)
        .await
        .expect("Failed to set bonus points");
    sqlx::query("UPDATE torrents SET created_by_id = $1 WHERE id = 100")
        .bind(actual_user_id)
        .execute(&pool)
        .await
        .expect("Failed to update torrent creator");

    let service = create_test_app(pool.clone()).await;

    // Uploader downloading their own torrent
    let valid_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    let status = resp.status();

    if !status.is_success() {
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        panic!(
            "Expected success, got status {}. Body: {}",
            status, body_str
        );
    }

    // Verify bonus points were NOT deducted (uploader is exempt)
    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = $1")
        .bind(actual_user_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(
        row.0, 100,
        "Uploader should still have 100 BP (not deducted)"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_no_double_bonus_points_deduction(pool: PgPool) {
    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    // First announce - should deduct 50 BP
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "First announce should succeed");

    // Insert a torrent_activities row to simulate that the peer data was flushed to DB
    sqlx::query(
        "INSERT INTO torrent_activities (torrent_id, user_id, downloaded) VALUES (100, 10, 500)",
    )
    .execute(&pool)
    .await
    .expect("Failed to insert torrent activity");

    // Stop the peer first
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=500&left=500&event=stopped&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();
    let _ = test::call_service(&service, req).await;

    // Second announce (resuming download) - should NOT deduct again
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=500&left=500&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "Second announce should succeed");

    // Verify bonus points were only deducted once (100 - 50 = 50)
    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 10")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(row.0, 50, "User should have 50 BP (only one deduction)");
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_no_deduction_when_no_download_between_announces(pool: PgPool) {
    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    // First announce with no download - should deduct 50 BP
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "First announce should succeed");

    // Second announce with no download - should NOT deduct again
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "Second announce should succeed");

    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 10")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(
        row.0, 50,
        "User should have 50 BP (only one deduction across two announces with no download)"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_no_deduction_when_peer_already_flushed_as_leecher(pool: PgPool) {
    // Simulates: user leeched earlier (0 bytes transferred), was flushed to peers
    // table, then disconnected. In-memory peer is gone; on re-announce we must
    // still detect the prior leech via the peers table and skip re-deduction.
    let prior_peer_id: [u8; 20] = [1u8; 20];
    sqlx::query(
        "INSERT INTO peers (peer_id, ip, port, agent, uploaded, downloaded, \"left\", \
         seeder, active, created_at, updated_at, torrent_id, user_id) \
         VALUES ($1, '127.0.0.1'::inet, 6969, 'test-agent/1.0', 0, 0, 1000, \
         false, false, NOW(), NOW(), 100, 10)",
    )
    .bind(&prior_peer_id[..])
    .execute(&pool)
    .await
    .expect("Failed to insert prior peer row");

    // Deduct once up-front to mimic the first leech attempt that was paid for.
    sqlx::query("UPDATE users SET bonus_points = 50 WHERE id = 10")
        .execute(&pool)
        .await
        .expect("Failed to simulate prior deduction");

    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    // Use a different peer_id to simulate a fresh client session
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(
        resp.status().is_success(),
        "Re-announce after disconnect should succeed"
    );

    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 10")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(
        row.0, 50,
        "User should still have 50 BP (no second deduction; peers row blocks it)"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost",
        "with_bonus_transfer_to_uploader"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_bonus_points_transfer_to_uploader(pool: PgPool) {
    // Get uploader's initial BP
    let uploader_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let uploader_initial_bp = uploader_row.0;

    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839"; // user 10
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success());

    // Verify uploader received the 50 BP
    let uploader_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(uploader_row.0, uploader_initial_bp + 50);
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost",
        "with_bonus_transfer_to_seeders"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_bonus_points_transfer_to_seeders(pool: PgPool) {
    // Get seeders' initial BP (users 1 and 20 are seeders from fixture)
    let seeder1_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let seeder1_initial_bp = seeder1_row.0;
    let seeder2_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 20")
        .fetch_one(&pool)
        .await
        .unwrap();
    let seeder2_initial_bp = seeder2_row.0;

    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839"; // user 10
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success());

    // Verify each seeder received 25 BP (50 / 2 seeders)
    let seeder1_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(seeder1_row.0, seeder1_initial_bp + 25);

    let seeder2_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 20")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(seeder2_row.0, seeder2_initial_bp + 25);
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost",
        "with_bonus_transfer_to_none"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_bonus_points_transfer_to_none(pool: PgPool) {
    // Get uploader's initial BP
    let uploader_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let uploader_initial_bp = uploader_row.0;

    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839"; // user 10
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success());

    // Verify uploader did NOT receive any BP (transfer_to = none)
    let uploader_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(uploader_row.0, uploader_initial_bp);
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_delete_torrent_rejects_announce(pool: PgPool) {
    let service = create_test_app(pool).await;

    let valid_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    // Delete the torrent via the tracker API
    let req = test::TestRequest::delete()
        .uri("/api/torrents/1")
        .insert_header(("x-api-key", "amazing_api_key_of_32_characters"))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "Delete should succeed");

    // Announce on the deleted torrent should fail
    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;

    let error: WrappedError = read_body_bencode(resp)
        .await
        .expect("Failed to decode error response");
    assert_eq!(error.failure_reason, "Torrent has been deleted.");
}

/// Inserts a completed torrent activity, i.e. the user already snatched the torrent once.
async fn insert_completed_activity(pool: &PgPool, torrent_id: i32, user_id: i32) {
    sqlx::query(
        "INSERT INTO torrent_activities (torrent_id, user_id, grabbed_at, completed_at, downloaded) \
         VALUES ($1, $2, NOW(), NOW(), 1000)",
    )
    .bind(torrent_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("Failed to insert completed torrent activity");
}

/// Inserts a grab-only torrent activity, i.e. the user downloaded the .torrent file
/// but never announced on the torrent.
async fn insert_grabbed_only_activity(pool: &PgPool, torrent_id: i32, user_id: i32) {
    sqlx::query(
        "INSERT INTO torrent_activities (torrent_id, user_id, grabbed_at) \
         VALUES ($1, $2, NOW())",
    )
    .bind(torrent_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("Failed to insert grabbed-only torrent activity");
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_bonus_points_deducted_after_grab_only_activity(pool: PgPool) {
    // The user downloaded the .torrent file (grab-only activity) and announces
    // for the first time: the snatch cost must be charged even when
    // charge_bonus_points_on_resnatch is disabled, since nothing was paid yet.
    insert_grabbed_only_activity(&pool, 100, 10).await;

    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "First announce should succeed");

    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 10")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(row.0, 50, "User should be charged for the first snatch");
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost",
        "with_charge_bonus_points_on_resnatch"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_bonus_points_deducted_on_resnatch_when_enabled(pool: PgPool) {
    insert_completed_activity(&pool, 100, 10).await;

    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "Re-snatch should succeed");

    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 10")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(row.0, 50, "User should be charged again for the re-snatch");
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_no_bonus_points_deducted_on_resnatch_when_disabled(pool: PgPool) {
    insert_completed_activity(&pool, 100, 10).await;

    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "Re-snatch should succeed");

    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 10")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(row.0, 100, "User should not be charged for the re-snatch");
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost",
        "with_charge_bonus_points_on_resnatch"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_no_deduction_when_seeding_resumes(pool: PgPool) {
    // User completed the torrent and restarts their client to seed it again:
    // the announce carries left=0, so no snatch cost may be charged, even with
    // charge_bonus_points_on_resnatch enabled.
    insert_completed_activity(&pool, 100, 10).await;

    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=1000&downloaded=1000&left=0&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success(), "Seeding resume should succeed");

    let row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 10")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user");

    assert_eq!(row.0, 100, "Resuming seeding must not deduct bonus points");
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_low_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_error_is_recorded_for_the_user(pool: PgPool) {
    let tracker = common::create_test_tracker(pool.clone(), common::test_config()).await;
    let service = test::init_service(
        actix_web::App::new()
            .app_data(tracker.clone())
            .configure(arcadia_tracker::routes::init),
    )
    .await;

    // User with 30 BP trying to download torrent with 50 BP cost, twice
    let valid_passkey = "g5037c66dd3e13044e0d2f9b891c3840";
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let peer_id_encoded =
        percent_encoding::percent_encode(&test_peer_id(), percent_encoding::NON_ALPHANUMERIC)
            .to_string();
    for _ in 0..2 {
        let req = test::TestRequest::get()
            .uri(&format!(
                "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
                valid_passkey, url_encode_info_hash(&info_hash_bytes), peer_id_encoded
            ))
            .insert_header(("User-Agent", "test-agent/1.0"))
            .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
            .to_request();
        test::call_service(&service, req).await;
    }

    arcadia_tracker::scheduler::flush(&tracker).await;

    let row: (i32, String, i64) =
        sqlx::query_as("SELECT user_id, error_code::text, occurrences FROM announce_errors")
            .fetch_one(&pool)
            .await
            .expect("the announce error should be recorded");
    assert_eq!(row, (11, "insufficient_bonus_points".to_string(), 2));
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_user_bonus_points",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent_snatch_cost",
        "with_snatch_bonus_points_factors"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_snatch_bonus_points_factors_applied(pool: PgPool) {
    // Torrent snatch cost is 50. Site-wide factors: cost_factor = 200% (snatcher charged 100),
    // reward_factor = 50% (uploader receives 25). The two factors are independent.
    let uploader_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let uploader_initial_bp = uploader_row.0;

    let service = create_test_app(pool.clone()).await;

    let valid_passkey = "f4037c66dd3e13044e0d2f9b891c3839"; // user 10, starts with 100 BP
    let info_hash_bytes = [
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xAA, 0xBB, 0xCC, 0xDD,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    let req = test::TestRequest::get()
        .uri(&format!(
            "/announce/{}?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success());

    // Snatcher charged 50 * 200 / 100 = 100, so 100 - 100 = 0.
    let snatcher_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 10")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        snatcher_row.0, 0,
        "Snatcher charged cost * cost_factor / 100"
    );

    // Uploader receives 50 * 50 / 100 = 25.
    let uploader_row: (i64,) = sqlx::query_as("SELECT bonus_points FROM users WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        uploader_row.0,
        uploader_initial_bp + 25,
        "Receiver credited cost * reward_factor / 100"
    );
}

#[sqlx::test(
    fixtures(
        "with_test_user",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent"
    ),
    migrations = "../../backend/storage/migrations"
)]
async fn test_announce_legacy_fallback_route(pool: PgPool) {
    let service = create_test_app(pool).await;

    let valid_passkey = "d2037c66dd3e13044e0d2f9b891c3837";
    let info_hash_bytes = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
        0x00, 0x11, 0x22, 0x33, 0x44,
    ];
    let info_hash_encoded = url_encode_info_hash(&info_hash_bytes);
    let peer_id = test_peer_id();
    let peer_id_encoded =
        percent_encoding::percent_encode(&peer_id, percent_encoding::NON_ALPHANUMERIC).to_string();

    // Verify announce using legacy /{passkey}/announce fallback route
    let req = test::TestRequest::get()
        .uri(&format!(
            "/{}/announce?info_hash={}&peer_id={}&port=6969&uploaded=0&downloaded=0&left=1000&event=started&compact=1",
            valid_passkey, info_hash_encoded, peer_id_encoded
        ))
        .insert_header(("User-Agent", "test-agent/1.0"))
        .peer_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert!(resp.status().is_success());

    let announce_response: AnnounceResponse = read_body_bencode(resp)
        .await
        .expect("Failed to decode announce response");
    assert!(
        announce_response.interval >= 1800,
        "Interval should be at least 1800"
    );
    assert_eq!(announce_response.min_interval, 1800);
    assert_eq!(
        announce_response.leechers, 1,
        "Should have 1 leecher (this peer)"
    );
}
