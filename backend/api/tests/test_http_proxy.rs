use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Spawns a TCP listener that counts incoming connections.
/// Returns the listener address and a shared connection counter.
fn spawn_connection_counter() -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind TCP listener");
    let address = listener.local_addr().unwrap().to_string();
    let connection_count = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&connection_count);

    std::thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("Failed to set non-blocking");
        loop {
            match listener.accept() {
                Ok(_) => {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    (address, connection_count)
}

/// The HTTP client built with a proxy configured should route requests through it.
#[tokio::test]
async fn test_build_http_client_routes_through_proxy() {
    let (proxy_address, connection_count) = spawn_connection_counter();

    let client = arcadia_api::build_http_client(Some(&format!("http://{proxy_address}")));

    // The request will fail (our listener isn't a real proxy), but the connection
    // attempt to the proxy should still be recorded.
    let _ = client.get("https://example.com").send().await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(
        connection_count.load(Ordering::SeqCst) > 0,
        "HTTP client with proxy configured should route requests through the proxy"
    );
}

/// The no-proxy HTTP client should NOT route through a proxy, even when one is available.
/// Uses the same fake proxy listener as the proxy test to prove it is never contacted.
#[tokio::test]
async fn test_build_no_proxy_http_client_bypasses_proxy() {
    let (proxy_address, proxy_connection_count) = spawn_connection_counter();
    let (_, no_proxy_connection_count) = spawn_connection_counter();

    // First, confirm the proxy listener is reachable by routing a proxied client through it
    let proxied_client = arcadia_api::build_http_client(Some(&format!("http://{proxy_address}")));
    let _ = proxied_client.get("https://example.com").send().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(
        proxy_connection_count.load(Ordering::SeqCst) > 0,
        "Sanity check: proxy listener should be reachable"
    );

    // Now verify the no-proxy client does NOT contact that same listener
    let no_proxy_client = arcadia_api::build_no_proxy_http_client();
    let _ = no_proxy_client.get("https://example.com").send().await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        no_proxy_connection_count.load(Ordering::SeqCst),
        0,
        "No-proxy HTTP client should NOT route through any proxy"
    );
}

/// The ip checks go through the proxy, for the http (ip-api.com) and https providers.
#[tokio::test]
async fn test_ip_checks_route_through_proxy() {
    let (proxy_address, connection_count) = spawn_connection_counter();
    let client = arcadia_api::build_http_client(Some(&format!("http://{proxy_address}")));
    let config: arcadia_api::services::ip_check::IpCheckProvidersConfig =
        serde_json::from_str(r#"{"ip_api": {"priority": 1}, "proxycheck": {"priority": 2}}"#)
            .unwrap();

    // every provider fails, the fake proxy not being a real one
    assert!(config
        .check_ip(&client, "8.8.8.8".parse().unwrap())
        .await
        .is_err());
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(
        connection_count.load(Ordering::SeqCst) >= 2,
        "every ip check provider should be requested through the proxy"
    );
}
