use crate::{
    middlewares::api_key_scopes::{is_endpoint_allowed_for_scopes, requires_no_authentication},
    Arcadia,
};
use actix_web::{
    dev::{Payload, ServiceRequest},
    error::ErrorUnauthorized,
    web::Data,
    Error, FromRequest, HttpMessage as _, HttpRequest,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use arcadia_storage::{models::user::Claims, redis::RedisPoolInterface};
use futures_util::future::{err, ok, Ready};
use jsonwebtoken::{decode, errors::ErrorKind, DecodingKey, Validation};
use subtle::ConstantTimeEq;

#[derive(Debug, Clone)]
pub struct Authdata {
    pub sub: i32,
}

impl FromRequest for Authdata {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        req.extensions()
            .get::<Authdata>()
            .cloned()
            .map(ok)
            .unwrap_or_else(|| err(ErrorUnauthorized("not authorized")))
    }
}

/// Returns the path the router will match the request against, which is percent decoded.
/// [`ServiceRequest::path`] returns the raw path instead, so matching on it would let
/// `/api/users/api%2Dkeys` dodge the checks below while still reaching the
/// `/api/users/api-keys` handler.
fn routed_path(req: &ServiceRequest) -> &str {
    req.match_info().as_str()
}

pub async fn authenticate_user<R: RedisPoolInterface + 'static>(
    req: ServiceRequest,
    bearer: Option<BearerAuth>,
) -> std::result::Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    // These routes are explicitly not authenticated.
    if requires_no_authentication(req.method(), routed_path(&req)) {
        return Ok(req);
    }

    // it is a request from a user
    if let Some(bearer) = bearer {
        validate_bearer_auth::<R>(req, bearer).await
    } else if let Some(api_key) = req.headers().get("api-key") {
        let api_key = api_key.to_str().expect("api_key malformed").to_owned();
        if routed_path(&req).starts_with("/api/tracker") {
            // it is a request from the tracker
            validate_tracker_api_key::<R>(req, &api_key)
        } else {
            validate_user_api_key::<R>(req, &api_key).await
        }
    } else {
        Err((
            ErrorUnauthorized("authentication error, missing jwt token or API key"),
            req,
        ))
    }
}

pub async fn validate_token<R: RedisPoolInterface + 'static>(
    token: &str,
    arc: &Data<Arcadia<R>>,
) -> Result<i32, Error> {
    let decoding_key = DecodingKey::from_secret(arc.api.jwt_secret.as_ref());
    let validation = Validation::default();

    let token_data =
        decode::<Claims>(token, &decoding_key, &validation).map_err(|err| match err.kind() {
            ErrorKind::ExpiredSignature => ErrorUnauthorized("jwt token expired"),
            _ => ErrorUnauthorized("authentication error"),
        })?;

    let user_id = token_data.claims.sub;

    let is_invalidated = arc
        .auth
        .is_invalidated(user_id, token_data.claims.iat)
        .await
        .map_err(|e| ErrorUnauthorized(e.to_string()))?;

    if is_invalidated {
        return Err(ErrorUnauthorized("token invalidated"));
    }

    Ok(user_id)
}

async fn validate_bearer_auth<R: RedisPoolInterface + 'static>(
    req: ServiceRequest,
    bearer: BearerAuth,
) -> std::result::Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let arc = req.app_data::<Data<Arcadia<R>>>().expect("app data set");

    let user_id = match validate_token::<R>(bearer.token(), arc).await {
        Ok(user_id) => user_id,
        Err(e) => return Err((e, req)),
    };

    let _ = arc.pool.update_last_seen_and_streak(user_id).await;
    req.extensions_mut().insert(Authdata { sub: user_id });

    Ok(req)
}

async fn validate_user_api_key<R: RedisPoolInterface + 'static>(
    req: ServiceRequest,
    api_key: &str,
) -> std::result::Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let arc = req.app_data::<Data<Arcadia<R>>>().expect("app data set");

    let (user_id, scopes) = match arc.pool.find_user_id_and_scopes_with_api_key(api_key).await {
        Ok(user_id_and_scopes) => user_id_and_scopes,
        Err(e) => return Err((actix_web::error::ErrorUnauthorized(e.to_string()), req)),
    };

    if !is_endpoint_allowed_for_scopes(req.method(), routed_path(&req), &scopes) {
        return Err((
            actix_web::error::ErrorForbidden(
                arcadia_common::error::Error::APIKeyScopeNotAllowed.to_string(),
            ),
            req,
        ));
    }

    req.extensions_mut().insert(Authdata { sub: user_id });

    Ok(req)
}

fn validate_tracker_api_key<R: RedisPoolInterface + 'static>(
    req: ServiceRequest,
    api_key: &str,
) -> std::result::Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let arc = req.app_data::<Data<Arcadia<R>>>().expect("app data set");

    if !bool::from(arc.tracker.api_key.as_bytes().ct_eq(api_key.as_bytes())) {
        return Err((actix_web::error::ErrorUnauthorized("invalid api key"), req));
    };

    Ok(req)
}
