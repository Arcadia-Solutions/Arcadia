use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RemoveTitleGroupFromSeriesRequest {
    pub series_id: i64,
    pub title_group_id: i32,
}

#[utoipa::path(
    delete,
    operation_id = "Remove title group from series",
    tag = "Series",
    path = "/api/series/title-group",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully removed the title group from the series"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<RemoveTitleGroupFromSeriesRequest>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::RemoveTitleGroupFromSeries,
            req.path(),
        )
        .await?;

    arc.pool
        .unassign_title_group_from_series(form.title_group_id, form.series_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
