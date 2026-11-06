use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
	put,
	operation_id = "Resolve staff PM",
	tag = "StaffPM",
	path = "/api/staff-pms/{id}/resolve",
	params(("id" = i64, Path, description = "Staff PM id")),
	security(("http" = ["Bearer"])) ,
	responses((status = 200, description = "Resolved staff PM", body = arcadia_storage::models::staff_pm::StaffPm))
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    id: Path<i64>,
) -> Result<HttpResponse> {
    let can_resolve_staff_pm = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::ResolveStaffPm)
        .await?;
    let staff_pm_id = id.into_inner();
    let updated = arc
        .pool
        .resolve_staff_pm(staff_pm_id, user.sub, can_resolve_staff_pm)
        .await?;
    arc.pool
        .mark_notifications_staff_pm_messages_as_read(staff_pm_id)
        .await?;
    Ok(HttpResponse::Ok().json(updated))
}
