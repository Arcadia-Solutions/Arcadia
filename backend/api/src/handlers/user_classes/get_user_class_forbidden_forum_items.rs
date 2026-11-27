use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{forum::UserClassForbiddenForumItems, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get user class forbidden forum items",
    tag = "User Class",
    path = "/api/user-classes/{name}/forbidden-forum-items",
    security(("http" = ["Bearer"])),
    params(
        ("name" = String, Path, description = "User class name")
    ),
    responses(
        (status = 200, description = "The forum items hidden from this user class", body=UserClassForbiddenForumItems),
        (status = 404, description = "User class not found"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    name: Path<String>,
    user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditUserClass, req.path())
        .await?;

    // fails with a not found error on an unknown class
    arc.pool.get_user_class_by_name(&name).await?;

    let forbidden_items = arc
        .pool
        .find_user_class_forbidden_forum_items(&name)
        .await?;

    Ok(HttpResponse::Ok().json(forbidden_items))
}
