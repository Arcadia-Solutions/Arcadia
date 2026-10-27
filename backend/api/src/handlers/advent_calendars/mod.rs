pub mod create_advent_calendar;
pub mod delete_advent_calendar;
pub mod edit_advent_calendar;
pub mod get_advent_calendar;
pub mod get_published_advent_calendar;
pub mod list_advent_calendars;
pub mod open_advent_calendar_door;
pub mod reset_advent_calendar_progress;

use actix_web::web::{delete, get, post, put, resource, ServiceConfig};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("")
            .route(get().to(self::list_advent_calendars::exec::<R>))
            .route(post().to(self::create_advent_calendar::exec::<R>)),
    );
    // registered before "/{id}" so that "published" is not parsed as an id
    cfg.service(
        resource("/published").route(get().to(self::get_published_advent_calendar::exec::<R>)),
    );
    cfg.service(
        resource("/doors/{door_id}/open")
            .route(post().to(self::open_advent_calendar_door::exec::<R>)),
    );
    cfg.service(
        resource("/{id}/progress")
            .route(delete().to(self::reset_advent_calendar_progress::exec::<R>)),
    );
    cfg.service(
        resource("/{id}")
            .route(get().to(self::get_advent_calendar::exec::<R>))
            .route(put().to(self::edit_advent_calendar::exec::<R>))
            .route(delete().to(self::delete_advent_calendar::exec::<R>)),
    );
}
