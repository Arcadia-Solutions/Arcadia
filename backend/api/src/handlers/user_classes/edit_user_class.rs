use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::user::{EditedUserClass, UserClass, UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit user class",
    tag = "User Class",
    path = "/api/user-classes/{name}",
    security(("http" = ["Bearer"])),
    params(
        ("name" = String, Path, description = "User class name")
    ),
    responses(
        (status = 200, description = "Successfully updated user class", body=UserClass),
        (status = 403, description = "Insufficient privileges"),
        (status = 404, description = "User class not found"),
        (status = 400, description = "Invalid user class name"),
        (status = 409, description = "User class name already exists"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    name: Path<String>,
    form: Json<EditedUserClass>,
    user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditUserClass, req.path())
        .await?;

    // name should be 3-30 characters
    if form.name.len() < 3 || form.name.len() > 30 {
        return Err(Error::InvalidUserClassName);
    }

    let user_class = arc.pool.update_user_class(&name, &form).await?;

    // update the settings cache if the default class name has changed
    if *name != form.name && *name == arc.settings.lock().unwrap().user_class_name_on_signup {
        let updated_settings = arc.pool.get_arcadia_settings().await?;
        *arc.settings.lock().unwrap() = updated_settings;
    }

    Ok(HttpResponse::Ok().json(user_class))
}
