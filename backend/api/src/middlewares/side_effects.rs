use crate::Arcadia;
use crate::middlewares::auth_middleware::Authdata;
use actix_http::body::to_bytes;
use actix_web::{
    HttpMessage as _, HttpResponse,
    body::{BoxBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::Method,
    middleware::Next,
    web::Data,
};
use arcadia_storage::models::arcadia_settings::HttpMethod;
use arcadia_storage::models::bonus_points_log::BonusPointsLogAction;
use arcadia_storage::redis::RedisPoolInterface;
use rand::RngExt as _;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum SideEffect {
    #[serde(rename = "bonus_points")]
    BonusPoints { amount: i64 },
}

pub async fn side_effects_middleware<R: RedisPoolInterface + 'static>(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error> {
    let user_id = req.extensions().get::<Authdata>().map(|a| a.sub);
    let method = match *req.method() {
        Method::GET => Some(HttpMethod::Get),
        Method::POST => Some(HttpMethod::Post),
        Method::PUT => Some(HttpMethod::Put),
        Method::PATCH => Some(HttpMethod::Patch),
        Method::DELETE => Some(HttpMethod::Delete),
        _ => None,
    };
    let path = req.path().to_owned();
    let arc = req.app_data::<Data<Arcadia<R>>>().cloned();

    let response = next.call(req).await?;

    if !response.status().is_success() || !is_json_response(&response) || path == "/api/auth/irc" {
        return Ok(response.map_into_boxed_body());
    }

    let side_effects = compute_side_effects(
        user_id,
        method.as_ref(),
        &path,
        arc.as_deref().map(|v| &**v),
    )
    .await;

    let status = response.status();
    let response = response.map_into_boxed_body();
    let (request, res) = response.into_parts();
    let body_bytes = to_bytes(res.into_body())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    let original: serde_json::Value =
        serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);

    let wrapped = serde_json::json!({
        "side_effects": side_effects,
        "data": original,
    });

    let new_response = HttpResponse::build(status).json(wrapped);

    Ok(ServiceResponse::new(request, new_response).map_into_boxed_body())
}

fn is_json_response<B>(response: &ServiceResponse<B>) -> bool {
    response
        .headers()
        .get("content-type")
        .and_then(|content_type| content_type.to_str().ok())
        .is_some_and(|content_type| content_type.contains("application/json"))
}

async fn compute_side_effects<R: RedisPoolInterface + 'static>(
    user_id: Option<i32>,
    method: Option<&HttpMethod>,
    path: &str,
    arc: Option<&Arcadia<R>>,
) -> Vec<SideEffect> {
    let mut side_effects = Vec::new();

    let Some(user_id) = user_id else {
        return side_effects;
    };
    let Some(arc) = arc else {
        return side_effects;
    };

    let configs = {
        let settings = arc.settings.lock().unwrap();
        settings.bonus_points_per_endpoint.clone()
    };

    let Some(method) = method else {
        return side_effects;
    };

    let Some(matched) = configs
        .0
        .iter()
        .find(|config| config.method == *method && path.starts_with(&config.path_prefix))
    else {
        return side_effects;
    };

    let roll: i16 = rand::rng().random_range(0..100);

    if roll < matched.probability {
        match arc
            .pool
            .award_bonus_points(
                user_id,
                BonusPointsLogAction::SideEffectReward,
                matched.amount,
                Some(path),
                None,
            )
            .await
        {
            Ok(()) => side_effects.push(SideEffect::BonusPoints {
                amount: matched.amount,
            }),
            Err(error) => log::error!(
                "Failed to award side-effect bonus points to user {}: {}",
                user_id,
                error
            ),
        }
    }

    side_effects
}
