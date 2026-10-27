use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{web::Data, HttpRequest, HttpResponse};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{advent_calendar::AdventCalendarLite, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "List advent calendars",
    tag = "Advent Calendars",
    path = "/api/advent-calendars",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully listed the advent calendars", body=Vec<AdventCalendarLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageAdventCalendars, req.path())
        .await?;

    let calendars = arc.pool.find_advent_calendars_lite().await?;
    Ok(HttpResponse::Ok().json(calendars))
}
