use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_shared::tracker::models::env::ArcadiaSettingsForTracker;
use log::info;

use crate::Tracker;

pub async fn exec(arc: Data<Tracker>, settings: Json<ArcadiaSettingsForTracker>) -> HttpResponse {
    info!("Updating settings: {:?}", *settings);

    *arc.settings.write() = settings.into_inner();

    HttpResponse::Ok().finish()
}
