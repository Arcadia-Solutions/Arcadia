pub mod create_user_class;
pub mod delete_user_class;
pub mod edit_user_class;
pub mod edit_user_class_forbidden_forum_items;
pub mod get_user_class_forbidden_forum_items;
pub mod get_user_classes;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_common::error::{Error, Result};
use arcadia_storage::redis::RedisPoolInterface;

/// 3-30 alphanumeric characters or underscores, as the name is used in urls
fn validate_user_class_name(name: &str) -> Result<()> {
    if name.len() < 3 || name.len() > 30 || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(Error::InvalidUserClassName);
    }
    Ok(())
}

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::get_user_classes::exec::<R>))
            .route(post().to(self::create_user_class::exec::<R>)),
    );
    cfg.service(
        resource("/{name}")
            .route(put().to(self::edit_user_class::exec::<R>))
            .route(delete().to(self::delete_user_class::exec::<R>)),
    );
    cfg.service(
        resource("/{name}/forbidden-forum-items")
            .route(get().to(self::get_user_class_forbidden_forum_items::exec::<R>))
            .route(put().to(self::edit_user_class_forbidden_forum_items::exec::<R>)),
    );
}
