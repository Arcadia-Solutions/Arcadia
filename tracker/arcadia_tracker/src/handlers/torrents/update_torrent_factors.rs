use actix_web::{
    HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_shared::tracker::models::torrent::APIUpdateTorrentFactors;
use log::info;

use crate::Tracker;

pub async fn exec(
    arc: Data<Tracker>,
    path: Path<u32>,
    factors: Json<APIUpdateTorrentFactors>,
) -> HttpResponse {
    let torrent_id = path.into_inner();

    info!(
        "Updating torrent {} factors: upload={}, download={}",
        torrent_id, factors.upload_factor, factors.download_factor
    );

    let mut found = false;
    arc.torrents.lock().entry(torrent_id).and_modify(|torrent| {
        torrent.upload_factor = factors.upload_factor;
        torrent.download_factor = factors.download_factor;
        found = true;
    });

    if found {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}
