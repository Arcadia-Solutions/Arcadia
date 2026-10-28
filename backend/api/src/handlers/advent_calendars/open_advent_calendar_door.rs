use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::advent_calendar::{AdventCalendarDoorOpeningAttempt, AdventCalendarDoorOpeningResult},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Open advent calendar door",
    tag = "Advent Calendars",
    path = "/api/advent-calendars/doors/{door_id}/open",
    security(
        ("http" = ["Bearer"])
    ),
    params(("door_id" = i32, Path, description = "Advent calendar door id")),
    request_body = AdventCalendarDoorOpeningAttempt,
    responses(
        (status = 200, description = "Successfully opened the door", body=AdventCalendarDoorOpeningResult),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i32>,
    payload: Json<AdventCalendarDoorOpeningAttempt>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let result = arc
        .pool
        .open_advent_calendar_door(user.sub, path.into_inner(), payload.passphrase.as_deref())
        .await?;

    Ok(HttpResponse::Ok().json(result))
}
