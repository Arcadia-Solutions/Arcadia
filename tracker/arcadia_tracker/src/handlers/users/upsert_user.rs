use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_shared::tracker::models::user::{APIInsertUser, User};
use log::{debug, info};

use crate::Tracker;

pub async fn exec(arc: Data<Tracker>, user: Json<APIInsertUser>) -> HttpResponse {
    info!("Inserting user with id {}.", user.id);

    // only insert, an already known user is left untouched
    arc.users.write().entry(user.id).or_insert(User {
        max_snatches_per_day: user.max_snatches_per_day,
        num_seeding: 0,
        num_leeching: 0,
        recent_leeches: Vec::new(),
    });

    arc.passkey2id.write().insert(user.passkey, user.id);

    debug!("inserted user: {:?}", user);

    HttpResponse::Ok().finish()
}
