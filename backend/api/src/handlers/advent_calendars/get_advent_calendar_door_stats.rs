use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        advent_calendar::{AdventCalendarDoorStats, AdventCalendarDoorStatsQuery},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get advent calendar door stats",
    tag = "Advent Calendars",
    path = "/api/advent-calendars/doors/{door_id}/stats",
    security(
        ("http" = ["Bearer"])
    ),
    params(
        ("door_id" = i32, Path, description = "Advent calendar door id"),
        AdventCalendarDoorStatsQuery
    ),
    responses(
        (status = 200, description = "Successfully got the door stats", body=AdventCalendarDoorStats),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i32>,
    query: Query<AdventCalendarDoorStatsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageAdventCalendars, req.path())
        .await?;

    let stats = arc
        .pool
        .find_advent_calendar_door_stats(path.into_inner(), &query)
        .await?;
    Ok(HttpResponse::Ok().json(stats))
}
