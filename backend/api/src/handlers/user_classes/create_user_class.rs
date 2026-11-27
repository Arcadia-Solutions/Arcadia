use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::user::{UserClass, UserCreatedUserClass, UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create user class",
    tag = "User Class",
    path = "/api/user-classes",
    security(("http" = ["Bearer"])),
    responses(
        (status = 201, description = "Successfully created user class", body=UserClass),
        (status = 403, description = "Insufficient privileges"),
        (status = 400, description = "Invalid user class name"),
        (status = 409, description = "User class already exists"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<UserCreatedUserClass>,
    user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::CreateUserClass, req.path())
        .await?;

    super::validate_user_class_name(&form.name)?;

    let user_class = arc.pool.create_user_class(&form).await?;

    Ok(HttpResponse::Created().json(user_class))
}
