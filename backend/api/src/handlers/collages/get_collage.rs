use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::collage::CollageEnriched, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetCollageQuery {
    pub id: i64,
}

#[utoipa::path(
    get,
    operation_id = "Get collage",
    tag = "Collages",
    path = "/api/collages",
    params(GetCollageQuery),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Collage information", body=CollageEnriched),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetCollageQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let collage = arc.pool.find_collage_enriched(query.id, user.sub).await?;
    arc.pool
        .mark_notification_collage_as_read(query.id, user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(collage))
}
