use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
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
    post,
    operation_id = "Create advent calendar",
    tag = "Advent Calendars",
    path = "/api/advent-calendars",
    security(
        ("http" = ["Bearer"])
    ),
    request_body = UserCreatedAdventCalendar,
    responses(
        (status = 200, description = "Successfully created the advent calendar", body=AdventCalendar),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    payload: Json<UserCreatedAdventCalendar>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageAdventCalendars, req.path())
        .await?;

    validate_advent_calendar(&payload)?;
    let created = arc.pool.create_advent_calendar(&payload, user.sub).await?;
    Ok(HttpResponse::Ok().json(created))
}
