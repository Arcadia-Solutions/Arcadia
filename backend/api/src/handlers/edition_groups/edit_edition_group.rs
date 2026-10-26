use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        edition_group::{EditedEditionGroup, EditionGroup},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

use crate::{
    middlewares::auth_middleware::Authdata, services::image_service::validate_image_urls, Arcadia,
};
use arcadia_common::error::{Error, Result};

#[utoipa::path(
    put,
    operation_id = "Edit edition group",
    tag = "Edition Group",
    path = "/api/edition-groups",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the edition group", body=EditionGroup),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<EditedEditionGroup>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let approved_image_hosts = arc.settings.lock().unwrap().approved_image_hosts.clone();
    validate_image_urls(&form.covers, &approved_image_hosts)?;

    let edition_group = arc.pool.find_edition_group(form.id).await?;

    if !arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditEditionGroup)
        .await?
        && edition_group.created_by_id != user.sub
    {
        return Err(Error::InsufficientPermissions(format!(
            "{:?}",
            UserPermission::EditEditionGroup
        )));
    }

    if let Some(edits) = compute_diff(&edition_group, &form, &["id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "edition_group".to_string(),
                item_id: edition_group.id as i64,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let updated_edition_group = arc.pool.update_edition_group(&form).await?;
    Ok(HttpResponse::Ok().json(updated_edition_group))
}
