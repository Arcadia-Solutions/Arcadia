use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::utils::compute_diff_with_allowed_users;
use arcadia_storage::{
    models::{
        forum::{EditedForumCategory, ForumCategory, ForumItemId},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit forum category",
    tag = "Forum",
    path = "/api/forum/category",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the forum category", body=ForumCategory),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    mut edited_category: Json<EditedForumCategory>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditForumCategory, req.path())
        .await?;

    let item = ForumItemId::Category(edited_category.id);
    arc.pool.ensure_forum_item_visible(user.sub, item).await?;

    let original_category = arc.pool.find_forum_category(edited_category.id).await?;
    // the allowed users are sorted by id, as the edited ones are before comparing them
    let original_allowed_user_ids: Vec<i32> = arc
        .pool
        .find_forum_item_allowed_users(item)
        .await?
        .iter()
        .map(|allowed_user| allowed_user.id)
        .collect();
    edited_category.allowed_user_ids.sort_unstable();
    edited_category.allowed_user_ids.dedup();
    let allowed_users_changed = original_allowed_user_ids != edited_category.allowed_user_ids;

    let updated_category = arc
        .pool
        .update_forum_category(&edited_category, allowed_users_changed, user.sub)
        .await?;

    if let Some(edits) = compute_diff_with_allowed_users(
        &original_category,
        &original_allowed_user_ids,
        &edited_category,
        &["id"],
    ) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "forum_category".to_string(),
                item_id: original_category.id as i64,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    Ok(HttpResponse::Ok().json(updated_category))
}
