use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{forum::ReorderForumCategories, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Reorder forum categories",
    tag = "Forum",
    path = "/api/forum/category/reorder",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully reordered the forum categories"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    reorder: Json<ReorderForumCategories>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditForumCategory, req.path())
        .await?;

    arc.pool.reorder_forum_categories(&reorder).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"result": "success"})))
}
