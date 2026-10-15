use actix_web::{web, HttpRequest, HttpResponse};
use arcadia_common::error::Error;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Serialize;

use crate::Arcadia;

use super::{authenticate_client, OAuthErrorResponse};

#[derive(Debug, Serialize)]
pub struct OAuthUserStatus {
    /// False when the user is banned or does not exist.
    pub active: bool,
}

/// Whether the user (the `sub` the client received from userinfo) may still use the client,
/// for live ban checks.
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: HttpRequest,
    user_id: web::Path<i32>,
    arc: web::Data<Arcadia<R>>,
) -> HttpResponse {
    // the client is configured by the admins, so it is trusted to check any user id
    if let Err(response) = authenticate_client(&request, &arc) {
        return response;
    }
    // only a missing user is inactive: a database failure must not end the user's sessions
    let active = match arc.pool.find_user_with_id(user_id.into_inner()).await {
        Ok(user) => !user.banned,
        Err(Error::UserWithIdNotFound(_)) => false,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(OAuthErrorResponse::new("server_error"))
        }
    };
    HttpResponse::Ok().json(OAuthUserStatus { active })
}
