use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json, Path},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        advent_calendar::{AdventCalendar, UserCreatedAdventCalendar},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
    services::advent_calendar_service::validate_advent_calendar,
};

#[utoipa::path(
    put,
    operation_id = "Edit advent calendar",
    tag = "Advent Calendars",
    path = "/api/advent-calendars/{id}",
    security(
        ("http" = ["Bearer"])
    ),
    params(("id" = i32, Path, description = "Advent calendar id")),
    request_body = UserCreatedAdventCalendar,
    responses(
        (status = 200, description = "Successfully edited the advent calendar", body=AdventCalendar),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i32>,
    payload: Json<UserCreatedAdventCalendar>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageAdventCalendars, req.path())
        .await?;

    validate_advent_calendar(&payload)?;
    let edited = arc
        .pool
        .update_advent_calendar(path.into_inner(), &payload)
        .await?;
    Ok(HttpResponse::Ok().json(edited))
}
