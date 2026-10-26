use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        title_group_tag::{EditedTitleGroupTag, TitleGroupTag},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit title group tag",
    tag = "Title Group Tag",
    path = "/api/title-group-tags",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the title group tag", body=TitleGroupTag),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    tag: Json<EditedTitleGroupTag>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditTitleGroupTag, req.path())
        .await?;

    let original_tag = arc.pool.find_title_group_tag(tag.id).await?;

    if let Some(edits) = compute_diff(&original_tag, &tag, &["id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "title_group_tag".to_string(),
                item_id: original_tag.id as i64,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let updated_tag = arc.pool.update_title_group_tag(&tag).await?;

    Ok(HttpResponse::Ok().json(updated_tag))
}
