use actix_web::{dev, web::Data, FromRequest, HttpRequest};
use arcadia_shared::tracker::models::{peer_id::PeerId, torrent::InfoHash};
use std::{
    borrow::Cow,
    future::{self, Ready},
    str::FromStr,
};

use crate::{announce::error::AnnounceError, Tracker};

#[derive(Debug)]
pub struct Announce {
    pub info_hash: InfoHash,
    pub peer_id: PeerId,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
    pub event: AnnounceEvent,
    pub numwant: usize,
    // corrupt: Option<u64>,
    // key: Option<String>,
    pub compact: Option<bool>,
}

impl FromRequest for Announce {
    type Error = AnnounceError;
    type Future = Ready<std::result::Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let query_string = req.query_string();

        let announce = decode_from_query_str(query_string, req);

        future::ready(announce)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AnnounceEvent {
    Started,
    Stopped,
    Completed,
    #[default]
    Empty,
}

impl FromStr for AnnounceEvent {
    type Err = AnnounceError;

    fn from_str(event: &str) -> Result<Self, Self::Err> {
        match event {
            "" | "empty" | "paused" => Ok(Self::Empty),
            "completed" => Ok(Self::Completed),
            "started" => Ok(Self::Started),
            "stopped" => Ok(Self::Stopped),
            _ => Err(AnnounceError::UnsupportedEvent),
        }
    }
}

struct QueryPairs<'a> {
    input: &'a str,
}

impl<'a> Iterator for QueryPairs<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.input.is_empty() {
                return None;
            }

            let mut split2 = self.input.splitn(2, '&');
            let sequence = split2.next().unwrap();
            self.input = split2.next().unwrap_or_default();
            if sequence.is_empty() {
                continue;
            }
            let mut split2 = sequence.splitn(2, '=');
            let name = split2.next().unwrap();
            let value = split2.next().unwrap_or_default();
            return Some((name, value));
        }
    }
}

struct IncorrectLength;

fn decode_into_20_byte_array(value: &str) -> Result<[u8; 20], IncorrectLength> {
    let x = Cow::from(percent_encoding::percent_decode_str(value));

    if x.len() != 20 {
        return Err(IncorrectLength);
    }

    Ok(std::array::from_fn(|i| unsafe {
        // SAFETY: length checked above.
        *x.get_unchecked(i)
    }))
}

pub fn decode_from_query_str(query: &str, req: &HttpRequest) -> Result<Announce, AnnounceError> {
    let mut info_hash = Option::<[u8; 20]>::None;
    let mut peer_id = Option::<[u8; 20]>::None;
    let mut port = Option::<u16>::None;
    let mut uploaded = Option::<u64>::None;
    let mut downloaded = Option::<u64>::None;
    let mut left = Option::<u64>::None;
    let mut event = Option::<AnnounceEvent>::None;
    let mut compact = Option::<bool>::None;
    let mut numwant = Option::<usize>::None;

    let pairs = QueryPairs { input: query };

    for (name, value) in pairs {
        match name {
            "info_hash" => {
                info_hash = Some(
                    decode_into_20_byte_array(value).map_err(|_| AnnounceError::InvalidInfoHash)?,
                );
            }
            "peer_id" => {
                peer_id = Some(
                    decode_into_20_byte_array(value).map_err(|_| AnnounceError::InvalidPeerId)?,
                );
            }
            "port" => {
                port = Some(u16::from_str(value).map_err(AnnounceError::InvalidPort)?);
            }
            "uploaded" => {
                uploaded = Some(u64::from_str(value).map_err(AnnounceError::InvalidUploaded)?);
            }
            "downloaded" => {
                downloaded = Some(u64::from_str(value).map_err(AnnounceError::InvalidDownloaded)?);
            }
            "left" => {
                left = Some(u64::from_str(value).map_err(AnnounceError::InvalidLeft)?);
            }
            "event" => {
                event =
                    Some(AnnounceEvent::from_str(value).map_err(|_| AnnounceError::InvalidEvent)?);
            }
            "compact" => match value {
                "1" => compact = Some(true),
                "0" => return Err(AnnounceError::UnsupportedCompact),
                _ => return Err(AnnounceError::InvalidCompact),
            },

            "numwant" => {
                numwant = Some(usize::from_str(value).map_err(AnnounceError::InvalidNumWant)?);
            }

            // key?
            _ => continue,
        };
    }

    let arc = req.app_data::<Data<Tracker>>().expect("app data set");

    Ok(Announce {
        info_hash: InfoHash(info_hash.ok_or(AnnounceError::MissingInfoHash)?),
        peer_id: PeerId(peer_id.ok_or(AnnounceError::MissingPeerId)?),
        port: port.ok_or(AnnounceError::MissingPort)?,
        uploaded: uploaded.ok_or(AnnounceError::MissingUploaded)?,
        downloaded: downloaded.ok_or(AnnounceError::MissingDownloaded)?,
        left: left.ok_or(AnnounceError::MissingLeft)?,
        event: event.unwrap_or_default(),
        compact,
        numwant: {
            if event.unwrap_or_default() == AnnounceEvent::Stopped {
                0
            } else {
                numwant.unwrap_or(arc.numwant_default).min(arc.numwant_max)
            }
        },
    })
}
