use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        title_group::{EditedTitleGroup, TitleGroup},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

use crate::{
    middlewares::auth_middleware::Authdata,
    services::image_service::{validate_image_urls, validate_trailer_urls},
    Arcadia,
};
use arcadia_common::error::{Error, Result};

#[utoipa::path(
    put,
    operation_id = "Edit title group",
    tag = "Title Group",
    path = "/api/title-groups",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the title group", body=TitleGroup),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<EditedTitleGroup>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let approved_image_hosts = arc.settings.lock().unwrap().approved_image_hosts.clone();
    validate_image_urls(&form.covers, &approved_image_hosts)?;
    validate_image_urls(&form.screenshots, &approved_image_hosts)?;
    validate_trailer_urls(&form.trailers)?;

    let title_group = arc.pool.find_title_group(form.id).await?;

    if !arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditTitleGroup)
        .await?
        && title_group.created_by_id != user.sub
    {
        return Err(Error::InsufficientPermissions(format!(
            "{:?}",
            UserPermission::EditTitleGroup
        )));
    }

    if let Some(edits) = compute_diff(&title_group, &form, &["id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "title_group".to_string(),
                item_id: title_group.id as i64,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let updated_title_group = arc.pool.update_title_group(&form, title_group.id).await?;
    Ok(HttpResponse::Ok().json(updated_title_group))
}
