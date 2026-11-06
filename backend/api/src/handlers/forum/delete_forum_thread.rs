use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{forum::DeleteForumThreadQuery, user::UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    delete,
    operation_id = "Delete forum thread",
    tag = "Forum",
    path = "/api/forum/thread",
    params(
        ("id" = i64, Query, description = "Forum thread ID to delete")
    ),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the forum thread and all its posts"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    delete_request: Query<DeleteForumThreadQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteForumThread, req.path())
        .await?;

    arc.pool.delete_forum_thread(delete_request.id).await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
