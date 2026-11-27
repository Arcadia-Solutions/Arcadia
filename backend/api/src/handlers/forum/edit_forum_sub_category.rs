use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::utils::compute_diff_with_allowed_users;
use arcadia_storage::{
    models::{
        forum::{EditedForumSubCategory, ForumItemId, ForumSubCategory},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit forum sub-category",
    tag = "Forum",
    path = "/api/forum/sub-category",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the forum sub-category", body=ForumSubCategory),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    mut edited_sub_category: Json<EditedForumSubCategory>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditForumSubCategory, req.path())
        .await?;

    let item = ForumItemId::SubCategory(edited_sub_category.id);
    arc.pool.ensure_forum_item_visible(user.sub, item).await?;

    let original_sub_category = arc
        .pool
        .find_forum_sub_category_raw(edited_sub_category.id)
        .await?;
    // the allowed users are sorted by id, as the edited ones are before comparing them
    let original_allowed_user_ids: Vec<i32> = arc
        .pool
        .find_forum_item_allowed_users(item)
        .await?
        .iter()
        .map(|allowed_user| allowed_user.id)
        .collect();
    edited_sub_category.allowed_user_ids.sort_unstable();
    edited_sub_category.allowed_user_ids.dedup();
    let allowed_users_changed = original_allowed_user_ids != edited_sub_category.allowed_user_ids;

    let change_log = compute_diff_with_allowed_users(
        &original_sub_category,
        &original_allowed_user_ids,
        &edited_sub_category,
        &["id"],
    )
    .map(|edits| NewUserEditChangeLog {
        item_type: "forum_sub_category".to_string(),
        item_id: original_sub_category.id as i64,
        edited_by_id: user.sub,
        edits,
    });

    let updated_sub_category = arc
        .pool
        .update_forum_sub_category(
            &edited_sub_category,
            allowed_users_changed,
            user.sub,
            change_log.as_ref(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(updated_sub_category))
}
