use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json, Path},
    HttpResponse,
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        title_group_comment::{EditedTitleGroupComment, TitleGroupComment},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit title group comment",
    tag = "Title Group",
    path = "/api/title-groups/comments/{id}",
    request_body = EditedTitleGroupComment,
    params(
        ("id" = i64, Path, description = "Comment id")
    ),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the comment", body = TitleGroupComment),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i64>,
    form: Json<EditedTitleGroupComment>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let comment_id = path.into_inner();

    let comment = arc.pool.find_title_group_comment(comment_id).await?;

    let is_staff = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditTitleGroupComment)
        .await?;
    let is_owner = comment.created_by_id == user.sub;

    if !is_staff && (!is_owner || comment.locked) {
        return Err(Error::InsufficientPermissions(format!(
            "{:?}",
            UserPermission::EditTitleGroupComment
        )));
    }

    if let Some(edits) = compute_diff(&comment, &form, &[]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "title_group_comment".to_string(),
                item_id: comment.id,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let updated_comment = arc
        .pool
        .update_title_group_comment(&form, comment_id)
        .await?;
    Ok(HttpResponse::Ok().json(updated_comment))
}
