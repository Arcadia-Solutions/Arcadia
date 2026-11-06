pub mod upload_image;

use actix_multipart::form::MultipartFormConfig;
use actix_web::web::{PayloadConfig, ServiceConfig, post, resource};
use arcadia_storage::redis::RedisPoolInterface;

// max file size: 40MB. the img host will block requests
// if its internal settings are set to a smaller value anyways
const MAX_UPLOAD_SIZE: usize = 40 * 1024 * 1024;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.app_data(PayloadConfig::default().limit(MAX_UPLOAD_SIZE))
        .app_data(
            MultipartFormConfig::default()
                .total_limit(MAX_UPLOAD_SIZE)
                .memory_limit(MAX_UPLOAD_SIZE),
        )
        .service(resource("/upload").route(post().to(self::upload_image::exec::<R>)));
}
