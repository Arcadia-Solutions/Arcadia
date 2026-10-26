use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        css_sheet::{CssSheet, EditedCssSheet},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit CSS sheet",
    tag = "Css Sheet",
    path = "/api/css-sheets",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the CSS sheet", body = CssSheet),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    css_sheet: Json<EditedCssSheet>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditCssSheet, req.path())
        .await?;

    let original_css_sheet = arc.pool.find_css_sheet(&css_sheet.old_name).await?;

    let old_name_was_default =
        css_sheet.old_name == arc.settings.lock().unwrap().default_css_sheet_name;

    let updated_css_sheet = arc.pool.update_css_sheet(&css_sheet).await?;
    // If the old name was the default, the CASCADE has already updated the database.
    // We just need to reload the settings from the database to update the in-memory cache.
    if old_name_was_default {
        let updated_settings = arc.pool.get_arcadia_settings().await?;
        *arc.settings.lock().unwrap() = updated_settings;
    }

    if let Some(edits) = compute_diff(&original_css_sheet, &css_sheet, &["old_name"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "css_sheet".to_string(),
                item_id: 0, // CSS sheets use name as identifier
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    Ok(HttpResponse::Ok().json(updated_css_sheet))
}
