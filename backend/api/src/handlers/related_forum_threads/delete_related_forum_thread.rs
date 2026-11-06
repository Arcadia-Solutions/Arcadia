use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{forum::DeleteRelatedForumThreadQuery, user::UserPermission},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    delete,
    operation_id = "Delete related forum thread",
    tag = "Related Forum Threads",
    path = "/api/related-forum-threads",
    params(DeleteRelatedForumThreadQuery),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully deleted the related forum thread"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<DeleteRelatedForumThreadQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::ManageRelatedForumThread,
            req.path(),
        )
        .await?;

    arc.pool.delete_related_forum_thread(&query).await?;

    Ok(HttpResponse::Ok().finish())
}
