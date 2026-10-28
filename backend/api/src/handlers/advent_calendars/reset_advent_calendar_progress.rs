use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Path},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Reset own advent calendar progress",
    tag = "Advent Calendars",
    path = "/api/advent-calendars/{id}/progress",
    security(
        ("http" = ["Bearer"])
    ),
    params(("id" = i32, Path, description = "Advent calendar id")),
    responses(
        (status = 200, description = "Successfully deleted the door openings of the current user"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i32>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageAdventCalendars, req.path())
        .await?;

    // meant for staff to test a calendar: the rewards already granted are kept,
    // so resetting and opening the doors again grants them once more
    arc.pool
        .delete_advent_calendar_door_openings_of_user(user.sub, path.into_inner())
        .await?;
    Ok(HttpResponse::Ok().finish())
}
