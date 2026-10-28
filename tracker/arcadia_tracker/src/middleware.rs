use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, web::Data};
use actix_web::{Error, HttpMessage};
use subtle::ConstantTimeEq;
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RequestId, RootSpanBuilder};

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

/// Same span as [`DefaultRootSpanBuilder`] without `http.target`: the announce
/// URL carries the passkey, which must not reach the logs nor the telemetry
/// collector. `http.route` (the templated path) is enough to identify the endpoint.
pub struct RootSpanBuilderWithoutTarget;

impl RootSpanBuilder for RootSpanBuilderWithoutTarget {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let connection_info = request.connection_info();
        let http_route = request.match_pattern().unwrap_or_else(|| "default".into());
        tracing::info_span!(
            "HTTP request",
            http.method = %request.method(),
            http.route = %http_route,
            http.flavor = ?request.version(),
            http.scheme = %connection_info.scheme(),
            http.host = %connection_info.host(),
            http.client_ip = %connection_info.realip_remote_addr().unwrap_or(""),
            http.user_agent = %request
                .headers()
                .get("User-Agent")
                .and_then(|h| h.to_str().ok())
                .unwrap_or(""),
            http.status_code = tracing::field::Empty,
            otel.name = %format!("{} {}", request.method(), http_route),
            otel.kind = "server",
            otel.status_code = tracing::field::Empty,
            request_id = %request.extensions().get::<RequestId>().copied().map(|id| id.to_string()).unwrap_or_default(),
            exception.message = tracing::field::Empty,
            exception.details = tracing::field::Empty,
        )
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
