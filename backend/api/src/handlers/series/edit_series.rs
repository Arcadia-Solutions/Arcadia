use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        series::{EditedSeries, Series},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit series",
    tag = "Series",
    path = "/api/series",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the series", body=Series),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<EditedSeries>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let series = arc.pool.find_series(&form.id).await?;

    if !arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditSeries)
        .await?
        && series.created_by_id != user.sub
    {
        return Err(Error::InsufficientPermissions(format!(
            "{:?}",
            UserPermission::EditSeries
        )));
    }

    if let Some(edits) = compute_diff(&series, &form, &["id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "series".to_string(),
                item_id: series.id,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let updated_series = arc.pool.update_series(&form).await?;

    Ok(HttpResponse::Ok().json(updated_series))
}
