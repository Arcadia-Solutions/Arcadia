use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        user::UserPermission,
        user_application::{UserApplication, UserApplicationStatus},
    },
    redis::RedisPoolInterface,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
pub struct UpdateUserApplication {
    pub status: UserApplicationStatus,
    pub user_application_id: i64,
}

#[utoipa::path(
    put,
    operation_id = "Update user application status",
    tag = "User Application",
    path = "/api/user-applications",
    request_body = UpdateUserApplication,
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully updated user application status", body = UserApplication),
        (status = 403, description = "Forbidden - Only staff members can update user applications"),
        (status = 404, description = "User application not found")
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    form: Json<UpdateUserApplication>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::UpdateUserApplication, req.path())
        .await?;

    let updated_application = arc
        .pool
        .update_user_application_status(form.user_application_id, form.status.clone())
        .await?;

    Ok(HttpResponse::Ok().json(updated_application))
}
