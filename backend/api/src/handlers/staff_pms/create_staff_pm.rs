use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::staff_pm::{StaffPm, UserCreatedStaffPm},
    redis::RedisPoolInterface,
};

#[utoipa::path(
	post,
	operation_id = "Create staff PM",
	tag = "StaffPM",
	path = "/api/staff-pms",
	security(("http" = ["Bearer"])) ,
	responses((status = 201, description = "Created staff PM conversation", body = StaffPm))
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    mut conversation: Json<UserCreatedStaffPm>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let created = arc
        .pool
        .create_staff_pm(&mut conversation, user.sub, &arc.notification_sender)
        .await?;
    Ok(HttpResponse::Created().json(created))
}
