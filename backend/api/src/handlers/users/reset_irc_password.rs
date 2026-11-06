use crate::{
    Arcadia, middlewares::auth_middleware::Authdata, services::irc_service::generate_irc_password,
};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::{Error, Result};
use arcadia_storage::redis::RedisPoolInterface;

use super::create_irc_account::IrcAccountResponse;

#[utoipa::path(
    put,
    operation_id = "Reset IRC password",
    tag = "User",
    path = "/api/users/irc",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully reset IRC password", body = IrcAccountResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    if !arc.ergo.is_enabled() {
        return Err(Error::IrcNotEnabled);
    }

    let current_user = arc.pool.find_user_with_id(user.sub).await?;

    if current_user.irc_password.is_none() {
        return Err(Error::IrcAccountNotFound);
    }

    // no need to call the ergo api for password change, the auth-script
    // delegates authentication to arcadia's api, so updating the password
    // in the db is sufficient.
    let irc_password = generate_irc_password();

    arc.pool.set_irc_password(user.sub, &irc_password).await?;

    Ok(HttpResponse::Ok().json(IrcAccountResponse { irc_password }))
}
