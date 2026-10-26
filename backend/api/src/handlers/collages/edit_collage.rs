use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{web::Data, web::Json, HttpResponse};
use arcadia_common::error::{Error, Result};
use arcadia_storage::models::collage::{Collage, EditedCollage};
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::models::user_edit_change_log::NewUserEditChangeLog;
use arcadia_storage::redis::RedisPoolInterface;
use arcadia_storage::utils::compute_diff;

#[utoipa::path(
    put,
    operation_id = "Edit collage",
    tag = "Collage",
    path = "/api/collages",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the collage", body=Collage),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<EditedCollage>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let collage = arc.pool.find_collage(form.id).await?;

    let has_permission = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditCollage)
        .await?;

    if !has_permission && collage.created_by_id != user.sub {
        return Err(Error::InsufficientPermissions(format!(
            "{:?}",
            UserPermission::EditCollage
        )));
    }

    if let Some(edits) = compute_diff(&collage, &form, &["id", "created_at", "created_by_id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "collage".to_string(),
                item_id: collage.id,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let updated_collage = arc.pool.update_collage(&form).await?;

    Ok(HttpResponse::Ok().json(updated_collage))
}
