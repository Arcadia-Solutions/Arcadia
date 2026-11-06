use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::models::staff_pm::StaffPmOverview;
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
	get,
	operation_id = "List staff PMs",
	tag = "StaffPM",
	path = "/api/staff-pms",
	security(("http" = ["Bearer"])) ,
	responses((status = 200, description = "List staff PM conversations", body = [StaffPmOverview]))
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let can_read_staff_pm = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::ReadStaffPm)
        .await?;
    let conversations = arc.pool.list_staff_pms(user.sub, can_read_staff_pm).await?;
    Ok(HttpResponse::Ok().json(conversations))
}
