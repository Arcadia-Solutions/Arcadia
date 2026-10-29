use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;
use serde::Deserialize;
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RemoveAffiliatedArtistsForm {
    pub affiliation_ids: Vec<i64>,
}

#[utoipa::path(
    delete,
    operation_id = "Delete artist affiliation",
    tag = "Affiliated Artist",
    path = "/api/affiliated-artists",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully removed the artist affiliations"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    form: Json<RemoveAffiliatedArtistsForm>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let title_group_ids = arc
        .pool
        .find_title_group_ids_of_artists_affiliation(&form.affiliation_ids)
        .await?;
    arc.pool
        .require_title_groups_edit_rights(user.sub, &title_group_ids, req.path())
        .await?;

    arc.pool
        .delete_artists_affiliation(&form.affiliation_ids)
        .await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
