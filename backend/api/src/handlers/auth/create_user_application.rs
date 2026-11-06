use crate::Arcadia;
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::user_application::{UserApplication, UserCreatedUserApplication},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create user application",
    tag = "User Application",
    path = "/api/auth/apply",
    responses(
        (status = 201, description = "Successfully created user application", body = UserApplication)
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
    application: Json<UserCreatedUserApplication>,
) -> Result<HttpResponse> {
    let client_ip = super::client_ip(
        &req,
        arc.config
            .api
            .reverse_proxy_client_ip_header_name
            .as_deref(),
    )?;

    let created_application = arc
        .pool
        .create_user_application(&application.into_inner(), client_ip)
        .await?;

    Ok(HttpResponse::Created().json(created_application))
}
