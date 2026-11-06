pub mod get_user_applications;
pub mod update_user_application_status;

use actix_web::web::{ServiceConfig, get, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::get_user_applications::exec::<R>))
            .route(put().to(self::update_user_application_status::exec::<R>)),
    );
}
