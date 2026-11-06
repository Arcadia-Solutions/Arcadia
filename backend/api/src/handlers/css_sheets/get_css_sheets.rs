use crate::Arcadia;
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{models::css_sheet::CssSheetsEnriched, redis::RedisPoolInterface};

#[utoipa::path(
    get,
    operation_id = "Get CSS sheets",
    tag = "Css Sheet",
    path = "/api/css-sheets",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the css sheets", body = CssSheetsEnriched),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(arc: Data<Arcadia<R>>) -> Result<HttpResponse> {
    let sheets = arc.pool.find_css_sheets().await?;
    Ok(HttpResponse::Ok().json(CssSheetsEnriched {
        css_sheets: sheets,
        default_sheet_name: arc.settings.lock().unwrap().default_css_sheet_name.clone(),
    }))
}
