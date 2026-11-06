use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::forum::ForumThreadEnrichedHierarchy, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetForumThreadQueryId {
    pub id: i64,
}

#[utoipa::path(
    get,
    operation_id = "Get forum thread",
    tag = "Forum",
    path = "/api/forum/thread",
    params(GetForumThreadQueryId),
    responses(
        (status = 200, description = "Returns the thread's information", body=ForumThreadEnrichedHierarchy)
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query_id: Query<GetForumThreadQueryId>,
    user: Authdata,
) -> Result<HttpResponse> {
    //TODO: restrict access to some sub_categories based on forbidden_classes

    let thread = arc
        .pool
        .find_forum_thread_with_poll(query_id.0.id, user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(thread))
}
