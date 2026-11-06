pub mod delete_all_user_edit_change_logs;
pub mod delete_user_edit_change_log;
pub mod search;

use actix_web::web::{ServiceConfig, delete, get, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::search::exec::<R>))
            .route(delete().to(self::delete_user_edit_change_log::exec::<R>)),
    )
    .service(
        resource("/all").route(delete().to(self::delete_all_user_edit_change_logs::exec::<R>)),
    );
}
