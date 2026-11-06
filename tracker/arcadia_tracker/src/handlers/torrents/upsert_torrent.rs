use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_shared::tracker::models::{
    peer,
    torrent::{APIInsertTorrent, Torrent},
};
use log::{debug, info};

use crate::Tracker;

pub async fn exec(arc: Data<Tracker>, torrent: Json<APIInsertTorrent>) -> HttpResponse {
    info!("Inserting torrent with id {}.", torrent.id);

    arc.torrents.lock().insert(
        torrent.id,
        Torrent {
            is_deleted: torrent.is_deleted,
            seeders: torrent.seeders,
            leechers: torrent.leechers,
            times_completed: torrent.times_completed,
            download_factor: torrent.download_factor as i16,
            upload_factor: torrent.upload_factor as i16,
            peers: peer::Map::new(),
        },
    );

    arc.infohash2id
        .write()
        .insert(torrent.info_hash, torrent.id);

    debug!("inserted torrent: {:?}", torrent);

    HttpResponse::Ok().finish()
}
