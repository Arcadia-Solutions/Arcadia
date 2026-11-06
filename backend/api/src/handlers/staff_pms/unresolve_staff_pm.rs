use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
	put,
	operation_id = "Unresolve staff PM",
	tag = "StaffPM",
	path = "/api/staff-pms/{id}/unresolve",
	params(("id" = i64, Path, description = "Staff PM id")),
	security(("http" = ["Bearer"])) ,
	responses((status = 200, description = "Unresolved staff PM", body = arcadia_storage::models::staff_pm::StaffPm))
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    id: Path<i64>,
) -> Result<HttpResponse> {
    let can_unresolve_staff_pm = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::UnresolveStaffPm)
        .await?;
    let updated = arc
        .pool
        .unresolve_staff_pm(id.into_inner(), user.sub, can_unresolve_staff_pm)
        .await?;
    Ok(HttpResponse::Ok().json(updated))
}
