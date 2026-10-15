use actix_web::{http::header::AUTHORIZATION, web, HttpRequest, HttpResponse};
use arcadia_storage::redis::RedisPoolInterface;
use serde::Serialize;

use crate::{services::oauth::find_by_token, Arcadia};

use super::OAuthErrorResponse;

#[derive(Debug, Serialize)]
pub struct UserinfoResponse {
    pub sub: String,
    pub preferred_username: String,
    pub email: String,
}

/// Claims of the user the access token was issued to.
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: HttpRequest,
    arc: web::Data<Arcadia<R>>,
) -> HttpResponse {
    // the bearer here is an opaque OAuth token, not an Arcadia JWT: it is looked up
    // in redis instead of going through the authentication middleware
    let invalid_token =
        || HttpResponse::Unauthorized().json(OAuthErrorResponse::new("invalid_token"));
    let Some(token) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
    else {
        return invalid_token();
    };
    let Ok(Some(user_id)) =
        find_by_token::<_, i32>(&*arc.redis_pool, "access_token", token.trim()).await
    else {
        return invalid_token();
    };
    // the only client is trusted with every claim, whatever scope it asked for
    match arc.pool.find_user_with_id(user_id).await {
        Ok(user) if !user.banned => HttpResponse::Ok().json(UserinfoResponse {
            sub: user.id.to_string(),
            preferred_username: user.username,
            email: user.email,
        }),
        _ => invalid_token(),
    }
}
