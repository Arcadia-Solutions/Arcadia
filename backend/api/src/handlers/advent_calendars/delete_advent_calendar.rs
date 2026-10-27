use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Path},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Delete advent calendar",
    tag = "Advent Calendars",
    path = "/api/advent-calendars/{id}",
    security(
        ("http" = ["Bearer"])
    ),
    params(("id" = i32, Path, description = "Advent calendar id")),
    responses(
        (status = 200, description = "Successfully deleted the advent calendar"),
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

    arc.pool.delete_advent_calendar(path.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}
