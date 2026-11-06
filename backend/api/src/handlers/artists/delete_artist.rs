use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data, web::Query};
use arcadia_common::error::Result;
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct DeleteArtistQuery {
    pub artist_id: i64,
}

#[utoipa::path(
    delete,
    operation_id = "Delete artist",
    tag = "Artist",
    path = "/api/artists",
    security(
        ("http" = ["Bearer"])
    ),
    params(DeleteArtistQuery),
    responses(
        (status = 200, description = "Successfully deleted the artist"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<DeleteArtistQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteArtist, req.path())
        .await?;

    arc.pool.delete_artist(query.artist_id).await?;

    Ok(HttpResponse::Ok().finish())
}
