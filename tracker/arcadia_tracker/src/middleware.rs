use actix_web::Error;
use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, web::Data};
use subtle::ConstantTimeEq;

use crate::Tracker;

pub async fn authenticate_backend(
    req: ServiceRequest,
    _: (),
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let api_key = &req
        .app_data::<Data<Tracker>>()
        .expect("app data set")
        .api_key;
    match req.headers().get("x-api-key").and_then(|v| v.to_str().ok()) {
        Some(k) if bool::from(k.as_bytes().ct_eq(api_key.as_bytes())) => Ok(req),
        _ => Err((ErrorUnauthorized("invalid or missing API key"), req)),
    }
}
