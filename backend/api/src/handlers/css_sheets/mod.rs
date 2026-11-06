pub mod create_css_sheet;
pub mod edit_css_sheet;
pub mod get_css_sheet;
pub mod get_css_sheet_content;
pub mod get_css_sheets;

use actix_web::web::{ServiceConfig, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(post().to(self::create_css_sheet::exec::<R>))
            .route(get().to(self::get_css_sheets::exec::<R>))
            .route(put().to(self::edit_css_sheet::exec::<R>)),
    );
    cfg.service(resource("/{name}").route(get().to(self::get_css_sheet::exec::<R>)));
}

pub fn config_public<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(resource("/{name}.css").route(get().to(self::get_css_sheet_content::exec::<R>)));
}
