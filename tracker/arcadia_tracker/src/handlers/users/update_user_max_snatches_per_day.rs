use actix_web::{
    HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_shared::tracker::models::user::APIUpdateUserMaxSnatchesPerDay;
use log::info;

use crate::Tracker;

pub async fn exec(
    arc: Data<Tracker>,
    path: Path<u32>,
    payload: Json<APIUpdateUserMaxSnatchesPerDay>,
) -> HttpResponse {
    let user_id = path.into_inner();

    info!(
        "Updating user {} max_snatches_per_day: {:?}",
        user_id, payload.max_snatches_per_day
    );

    let mut found = false;
    arc.users.write().entry(user_id).and_modify(|user| {
        user.max_snatches_per_day = payload.max_snatches_per_day;
        found = true;
    });

    if found {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}
