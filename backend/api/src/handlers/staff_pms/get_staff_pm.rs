use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::models::{staff_pm::StaffPmHierarchy, user::UserPermission};
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
	get,
	operation_id = "Get staff PM",
	tag = "StaffPM",
	path = "/api/staff-pms/{id}",
	params(("id" = i64, Path, description = "Staff PM id")),
	security(("http" = ["Bearer"])) ,
	responses((status = 200, description = "Staff PM conversation details", body = StaffPmHierarchy))
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    id: Path<i64>,
) -> Result<HttpResponse> {
    let staff_pm_id = id.into_inner();
    let can_read_staff_pm = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::ReadStaffPm)
        .await?;
    let conv = arc
        .pool
        .get_staff_pm(staff_pm_id, user.sub, can_read_staff_pm)
        .await?;
    arc.pool
        .mark_notification_staff_pm_message_as_read(staff_pm_id, user.sub)
        .await?;
    Ok(HttpResponse::Ok().json(conv))
}
