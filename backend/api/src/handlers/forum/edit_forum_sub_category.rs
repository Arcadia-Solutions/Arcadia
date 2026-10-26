use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        forum::{EditedForumSubCategory, ForumSubCategory},
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
    edited_sub_category: Json<EditedForumSubCategory>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditForumSubCategory, req.path())
        .await?;

    let original_sub_category = arc
        .pool
        .find_forum_sub_category_raw(edited_sub_category.id)
        .await?;

    if let Some(edits) = compute_diff(&original_sub_category, &edited_sub_category, &["id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "forum_sub_category".to_string(),
                item_id: original_sub_category.id as i64,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let updated_sub_category = arc
        .pool
        .update_forum_sub_category(&edited_sub_category)
        .await?;

    Ok(HttpResponse::Ok().json(updated_sub_category))
}
