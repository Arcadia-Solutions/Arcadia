use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use log::info;

use crate::Tracker;

pub async fn exec(arc: Data<Tracker>, path: Path<u32>) -> HttpResponse {
    let torrent_id = path.into_inner();

    info!("Marking torrent {torrent_id} as deleted.");

    if let Some(torrent) = arc.torrents.lock().get_mut(&torrent_id) {
        torrent.is_deleted = true;
    }

    HttpResponse::Ok().finish()
}
