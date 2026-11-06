use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{arcadia_settings::ArcadiaSettings, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get Arcadia settings",
    tag = "Arcadia Settings",
    path = "/api/arcadia-settings",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved Arcadia settings", body=ArcadiaSettings),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditArcadiaSettings, req.path())
        .await?;

    let settings = arc.settings.lock().unwrap().clone();
    Ok(HttpResponse::Ok().json(settings))
}
