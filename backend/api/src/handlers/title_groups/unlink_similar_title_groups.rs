use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{title_group::SimilarTitleGroupsLink, user::UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    delete,
    operation_id = "Unlink similar title groups",
    tag = "Title Group",
    path = "/api/title-groups/similar",
    request_body = SimilarTitleGroupsLink,
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully unlinked the title groups"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    link: Json<SimilarTitleGroupsLink>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::UnlinkSimilarTitleGroup,
            req.path(),
        )
        .await?;

    arc.pool.unlink_similar_title_groups(&link).await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
