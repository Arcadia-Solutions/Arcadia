use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{conversation::ConversationHierarchy, user::UserPermission},
    redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetConversationQuery {
    id: i64,
}

#[utoipa::path(
    get,
    operation_id = "Get conversation",
    tag = "Conversation",
    params(GetConversationQuery),
    path = "/api/conversations",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Found the conversation and its messages", body=ConversationHierarchy),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetConversationQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let can_read_all = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::ReadAllConversations)
        .await?;
    let conversation_with_messages = arc
        .pool
        .find_conversation(query.id, user.sub, can_read_all)
        .await?;

    Ok(HttpResponse::Ok().json(conversation_with_messages))
}
