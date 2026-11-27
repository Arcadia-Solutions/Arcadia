use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        forum::{EditedForumThread, ForumThreadEnriched},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit forum thread",
    tag = "Forum",
    path = "/api/forum/thread",
    responses(
        (status = 200, description = "Edits the thread's information", body=ForumThreadEnriched)
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    user: Authdata,
    edited_forum_thread: Json<EditedForumThread>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let original_thread = arc
        .pool
        .find_forum_thread(edited_forum_thread.id, user.sub)
        .await?;

    if !arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditForumThread)
        .await?
        && original_thread.created_by_id != user.sub
    {
        return Err(Error::from(UserPermission::EditForumThread));
    }

    // intentionally checked again: being the thread's author is enough to edit it,
    // but (un)locking requires the explicit edit_forum_thread permission
    if original_thread.locked != edited_forum_thread.locked {
        arc.pool
            .require_permission(user.sub, &UserPermission::EditForumThread, req.path())
            .await?;
    }

    let change_log = compute_diff(&original_thread, &edited_forum_thread, &["id"]).map(|edits| {
        NewUserEditChangeLog {
            item_type: "forum_thread".to_string(),
            item_id: original_thread.id,
            edited_by_id: user.sub,
            edits,
        }
    });

    let updated_thread = arc
        .pool
        .update_forum_thread(&edited_forum_thread, user.sub, change_log.as_ref())
        .await?;

    Ok(HttpResponse::Ok().json(updated_thread))
}
