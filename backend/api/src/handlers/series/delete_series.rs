use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data, web::Query};
use arcadia_common::error::Result;
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::redis::RedisPoolInterface;

use super::get_series::GetSeriesQuery;

#[utoipa::path(
    delete,
    operation_id = "Delete series",
    tag = "Series",
    path = "/api/series",
    security(
        ("http" = ["Bearer"])
    ),
    params(GetSeriesQuery),
    responses(
        (status = 200, description = "Successfully deleted the series"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetSeriesQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteSeries, req.path())
        .await?;

    arc.pool.delete_series(query.id).await?;

    Ok(HttpResponse::Ok().finish())
}
