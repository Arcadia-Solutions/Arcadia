use arcadia_shared::config::{
    default_log_level, deserialize_secret, DatabaseConfig, TelemetryConfig,
};
use serde::{Deserialize, Deserializer};
use std::collections::HashSet;

/// Sections of the `config.yml` file used by the tracker. The other ones are ignored.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub tracker: TrackerConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8081
}

fn default_numwant() -> usize {
    15
}

fn default_announce_min() -> u32 {
    1800
}

fn default_announce_min_enforced() -> u32 {
    1740
}

fn default_announce_max() -> u32 {
    3600
}

fn default_max_peers_per_torrent_per_user() -> u8 {
    3
}

fn default_flush_interval_milliseconds() -> u64 {
    3000
}

fn default_peer_expiry_interval() -> u64 {
    1800
}

fn default_active_peer_ttl() -> u64 {
    7200
}

fn default_inactive_peer_ttl() -> u64 {
    1814400
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerConfig {
    /// Address the tracker listens on.
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    /// Used for the backend to make requests to the tracker and vice-versa.
    #[serde(deserialize_with = "deserialize_secret")]
    pub api_key: String,
    /// Overridden by the `RUST_LOG` environment variable when it is set.
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Allowed torrent clients, without their leading "-".
    pub allowed_torrent_clients: AllowedTorrentClientSet,
    /// Amount of peers sent back when the peer does not include a numwant.
    #[serde(default = "default_numwant")]
    pub numwant_default: usize,
    /// Max amount of peers sent back when the peer's numwant is too high.
    #[serde(default = "default_numwant")]
    pub numwant_max: usize,
    /// A random amount of seconds between `announce_min` and `announce_max` is returned to the
    /// peer as the time of its next announce.
    #[serde(default = "default_announce_min")]
    pub announce_min: u32,
    /// Announcing before this many seconds after the previous announce returns a rate limit error.
    #[serde(default = "default_announce_min_enforced")]
    pub announce_min_enforced: u32,
    #[serde(default = "default_announce_max")]
    pub announce_max: u32,
    /// Max amount of active peers a user is allowed to have on a torrent.
    #[serde(default = "default_max_peers_per_torrent_per_user")]
    pub max_peers_per_torrent_per_user: u8,
    /// Interval between the flushes of history, peers, torrents and users to the database.
    #[serde(default = "default_flush_interval_milliseconds")]
    pub flush_interval_milliseconds: u64,
    /// Amount of seconds between the batches marking peers as inactive or erasing them.
    #[serde(default = "default_peer_expiry_interval")]
    pub peer_expiry_interval: u64,
    /// Amount of seconds since the last announce before a peer is considered inactive.
    #[serde(default = "default_active_peer_ttl")]
    pub active_peer_ttl: u64,
    /// Amount of seconds since the last announce before a peer is erased from memory. Long enough
    /// that users can suffer multi-day network outages without their stats being recorded wrong.
    #[serde(default = "default_inactive_peer_ttl")]
    pub inactive_peer_ttl: u64,
    /// Header set by the reverse proxy holding the original ip address of the bittorrent client.
    /// The last address of the comma separated list is selected. Leave unset to use the connecting
    /// ip address when there is no reverse proxy.
    #[serde(default)]
    pub reverse_proxy_client_ip_header_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AllowedTorrentClientSet {
    pub clients: HashSet<Vec<u8>>,
}

impl<'de> Deserialize<'de> for AllowedTorrentClientSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let clients = Vec::<String>::deserialize(deserializer)?
            .into_iter()
            .map(|client| client.trim().as_bytes().to_vec())
            .collect::<HashSet<Vec<u8>>>();

        Ok(Self { clients })
    }
}
