use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        conversation::{ConversationSearchQuery, ConversationSearchResult},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search conversations",
    tag = "Search",
    path = "/api/search/conversations",
    params(ConversationSearchQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Search conversations", body=PaginatedResults<ConversationSearchResult>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    req: HttpRequest,
    query: Query<ConversationSearchQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let all_conversations = query.all_conversations;

    if all_conversations {
        arc.pool
            .require_permission(user.sub, &UserPermission::ReadAllConversations, req.path())
            .await?;
    }

    let results = arc
        .pool
        .search_conversations(user.sub, &query, all_conversations)
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
