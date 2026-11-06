use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{forum::DeleteForumPostQuery, user::UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    delete,
    operation_id = "Delete forum post",
    tag = "Forum",
    path = "/api/forum/post",
    params(
        ("id" = i64, Query, description = "Forum post ID to delete")
    ),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the forum post"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    delete_request: Query<DeleteForumPostQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteForumPost, req.path())
        .await?;

    arc.pool.delete_forum_post(delete_request.id).await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
