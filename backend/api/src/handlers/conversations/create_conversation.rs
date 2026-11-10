use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::conversation::{Conversation, UserCreatedConversation},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create conversation",
    tag = "Conversation",
    path = "/api/conversations",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully created the conversation and first message", body=Conversation),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    mut conversation: Json<UserCreatedConversation>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    // creates a conversation and the first message, empty conversations should not be allowed
    let conversation = arc
        .pool
        .create_conversation(&mut conversation, user.sub)
        .await?;

    Ok(HttpResponse::Created().json(conversation))
}
