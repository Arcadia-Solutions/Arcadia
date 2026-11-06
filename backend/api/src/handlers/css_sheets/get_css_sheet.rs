use crate::Arcadia;
use actix_web::{HttpResponse, web::Data, web::Path};
use arcadia_common::error::Result;
use arcadia_storage::{models::css_sheet::CssSheet, redis::RedisPoolInterface};

#[utoipa::path(
    get,
    operation_id = "Get CSS sheet",
    tag = "Css Sheet",
    path = "/api/css-sheets/{name}",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the css sheet", body = CssSheet),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    name: Path<String>,
) -> Result<HttpResponse> {
    let sheet = arc.pool.find_css_sheet(&name).await?;
    Ok(HttpResponse::Ok().json(sheet))
}
