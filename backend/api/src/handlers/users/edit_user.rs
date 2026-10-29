use crate::{
    middlewares::auth_middleware::Authdata,
    services::{
        auth_service::{validate_email, verify_password},
        image_service::validate_image_url,
    },
    Arcadia,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{models::user::EditedUser, redis::RedisPoolInterface};
use serde_json::json;

#[utoipa::path(
    put,
    operation_id = "Edit user",
    tag = "User",
    path = "/api/users",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the user"),
        (status = 400, description = "Invalid email or wrong current password"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<EditedUser>,
    user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    if let Some(ref avatar) = form.avatar {
        let approved_image_hosts = arc.settings.lock().unwrap().approved_image_hosts.clone();
        validate_image_url(avatar, &approved_image_hosts)?;
    }

    let current_user = arc.pool.find_user_with_id(user.sub).await?;
    if form.email != current_user.email {
        validate_email(&form.email)?;
        verify_password(
            form.current_password
                .as_deref()
                .ok_or(Error::WrongUsernameOrPassword)?,
            &current_user.password_hash,
        )?;
    }

    arc.pool.update_user(user.sub, &form).await?;

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}
