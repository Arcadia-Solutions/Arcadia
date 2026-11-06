use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
    get,
    operation_id = "Get CSS sheet content",
    tag = "Css Sheet",
    path = "/api/css/{name}.css",
    responses(
        (status = 200, description = "Successfully retrieved the CSS content", content_type = "text/css"),
        (status = 404, description = "CSS sheet not found"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    name: Path<String>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let css = arc.pool.get_css_sheet_content(&name).await?;
    Ok(HttpResponse::Ok()
        .content_type("text/css")
        .insert_header((
            "Cache-Control",
            "no-store, no-cache, must-revalidate, max-age=0",
        ))
        .insert_header(("Pragma", "no-cache"))
        .body(css))
}
