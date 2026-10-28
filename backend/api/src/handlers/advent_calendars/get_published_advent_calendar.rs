use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{web::Data, HttpResponse};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::advent_calendar::PublishedAdventCalendar, redis::RedisPoolInterface,
    services::advent_calendar_service::build_published_advent_calendar,
};
use chrono::Utc;

#[utoipa::path(
    get,
    operation_id = "Get published advent calendar",
    tag = "Advent Calendars",
    path = "/api/advent-calendars/published",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the published advent calendar", body=PublishedAdventCalendar),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let calendar = arc.pool.find_published_advent_calendar().await?;
    let openings = arc
        .pool
        .find_advent_calendar_door_openings_of_user(user.sub, calendar.id)
        .await?;
    Ok(HttpResponse::Ok().json(build_published_advent_calendar(
        calendar,
        &openings,
        Utc::now(),
    )))
}
