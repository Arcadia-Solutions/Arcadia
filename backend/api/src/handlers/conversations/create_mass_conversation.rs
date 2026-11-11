use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        conversation::{MassMessageRequest, MassMessageResult},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create mass conversation",
    tag = "Conversation",
    path = "/api/conversations/mass",
    security(
      ("http" = ["Bearer"])
    ),
    description = "Sends a private message to every user matching the search filter (username, registration date range and/or permissions), across all pages. \
                   Filtering on permissions requires the set_and_view_user_permissions permission.",
    responses(
        (status = 200, description = "Successfully sent the message to every matching user", body=MassMessageResult),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    payload: Json<MassMessageRequest>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(current_user.sub, &UserPermission::SendMassPm, req.path())
        .await?;

    // Filtering on permissions reveals which permissions a user has, so it is only allowed for
    // staff members that may also set and view them.
    if payload
        .permissions
        .as_ref()
        .is_some_and(|permissions| !permissions.is_empty())
    {
        arc.pool
            .require_permission(
                current_user.sub,
                &UserPermission::SetAndViewUserPermissions,
                req.path(),
            )
            .await?;
    }

    let result = arc
        .pool
        .create_mass_conversation(&payload, current_user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(result))
}
