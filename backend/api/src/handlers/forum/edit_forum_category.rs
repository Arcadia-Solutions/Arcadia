use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        forum::{EditedForumCategory, ForumCategory},
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
    edited_category: Json<EditedForumCategory>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditForumCategory, req.path())
        .await?;

    let original_category = arc.pool.find_forum_category(edited_category.id).await?;

    if let Some(edits) = compute_diff(&original_category, &edited_category, &["id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "forum_category".to_string(),
                item_id: original_category.id as i64,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let updated_category = arc.pool.update_forum_category(&edited_category).await?;

    Ok(HttpResponse::Ok().json(updated_category))
}
