use anyhow::{bail, Context};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgTypeInfo;
use sqlx::{Database, Decode, PgPool, Postgres, Type};
use std::net::IpAddr;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use utoipa::ToSchema;

use crate::tracker::models::peer::{self, Peer};
use crate::tracker::models::peer_id::PeerId;
use crate::utils::hex_decode;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, Hash, PartialEq, ToSchema)]
pub struct InfoHash(pub [u8; 20]);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Torrent {
    pub upload_factor: i16,
    pub download_factor: i16,
    pub seeders: u32,
    pub leechers: u32,
    pub times_completed: u32,
    pub is_deleted: bool,
    pub peers: peer::Map,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct APIInsertTorrent {
    pub id: u32,
    pub info_hash: InfoHash,
    pub is_deleted: bool,
    pub seeders: u32,
    pub leechers: u32,
    pub times_completed: u32,
    pub download_factor: u8,
    pub upload_factor: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct APIUpdateTorrentFactors {
    pub upload_factor: i16,
    pub download_factor: i16,
}

#[derive(Debug)]
pub struct Map(pub IndexMap<u32, Torrent>);

#[derive(Debug)]
pub struct DBImportTorrent {
    pub id: i32,
    pub upload_factor: i16,
    pub download_factor: i16,
    pub seeders: i64,
    pub leechers: i64,
    pub times_completed: i32,
    pub is_deleted: bool,
}

impl Map {
    pub async fn from_database(db: &PgPool) -> Self {
        let rows = sqlx::query_as!(
            DBImportTorrent,
            r#"
                    SELECT
                        id,
                        upload_factor,
                        download_factor,
                        seeders,
                        leechers,
                        times_completed,
                        CASE
                            WHEN deleted_at IS NOT NULL THEN TRUE
                            ELSE FALSE
                        END AS "is_deleted!"
                    FROM torrents
                    "#
        )
        .fetch_all(db)
        .await
        .expect("could not get torrents");

        let mut map: Map = Map(IndexMap::with_capacity(rows.len()));
        for r in rows {
            let torrent = Torrent {
                upload_factor: r.upload_factor,
                download_factor: r.download_factor,
                seeders: r.seeders as u32,
                leechers: r.leechers as u32,
                times_completed: r.times_completed as u32,
                is_deleted: r.is_deleted,
                peers: peer::Map::new(),
            };
            map.insert(r.id as u32, torrent);
        }

        // Load peers into each torrent
        let peers = sqlx::query!(
            r#"
                SELECT
                    peers.ip AS "ip_address: IpAddr",
                    peers.user_id AS "user_id",
                    peers.torrent_id AS "torrent_id",
                    peers.port AS "port",
                    peers.seeder AS "is_seeder: bool",
                    peers.active AS "is_active: bool",
                    peers.updated_at AS "updated_at: DateTime<Utc>",
                    peers.uploaded AS "uploaded",
                    peers.downloaded AS "downloaded",
                    peers.peer_id AS "peer_id: PeerId"
                FROM peers
            "#
        )
        .fetch_all(db)
        .await
        .expect("Failed loading peers from database");

        for peer in peers {
            let torrent_id =
                u32::try_from(peer.torrent_id).expect("torrent_id out of range for u32");
            let user_id = u32::try_from(peer.user_id).expect("user_id out of range for u32");
            #[expect(clippy::expect_fun_call)]
            let port = u16::try_from(peer.port).expect(&format!(
                "Invalid port number in database. Peer: {:?}",
                peer
            ));

            map.entry(torrent_id).and_modify(|torrent| {
                torrent.peers.insert(
                    peer::Index {
                        user_id,
                        peer_id: peer.peer_id,
                    },
                    Peer {
                        ip_address: peer.ip_address,
                        port,
                        is_seeder: peer.is_seeder,
                        is_active: peer.is_active,
                        has_sent_completed: false,
                        updated_at: peer
                            .updated_at
                            .expect("Peer with null updated_at found in database."),
                        uploaded: peer.uploaded as u64,
                        downloaded: peer.downloaded as u64,
                    },
                );
            });
        }

        map
    }
}

impl Deref for Map {
    type Target = IndexMap<u32, Torrent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Type<Postgres> for InfoHash {
    fn type_info() -> PgTypeInfo {
        <Vec<u8> as Type<Postgres>>::type_info()
    }
}

impl<'r, DB: Database> Decode<'r, DB> for InfoHash
where
    &'r [u8]: Decode<'r, DB>,
{
    /// Decodes the database's string representation of the 40 character long
    /// infohash in hex into a byte slice
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<InfoHash, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <&[u8] as Decode<DB>>::decode(value)?;

        if value.len() != 20 {
            let error: Box<dyn std::error::Error + Send + Sync> =
                Box::new(crate::error::DecodeError::InfoHash);

            return Err(error);
        }

        Ok(InfoHash(<[u8; 20]>::try_from(&value[0..20])?))
    }
}

impl FromStr for InfoHash {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        let mut out = [0u8; 20];

        if bytes.len() != 40 {
            bail!("`{s}` is not a valid infohash.");
        }

        for pos in 0..20 {
            out[pos] = hex_decode([bytes[pos * 2], bytes[pos * 2 + 1]])
                .context("`{s}` is not a valid infohash")?;
        }

        Ok(InfoHash(out))
    }
}
