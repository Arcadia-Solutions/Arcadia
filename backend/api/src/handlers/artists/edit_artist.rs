use crate::{
    middlewares::auth_middleware::Authdata, services::image_service::validate_image_urls, Arcadia,
};
use actix_web::{web::Data, web::Json, HttpResponse};
use arcadia_common::error::{Error, Result};
use arcadia_storage::models::artist::Artist;
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::models::user_edit_change_log::NewUserEditChangeLog;
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{models::artist::EditedArtist, redis::RedisPoolInterface};

const GRACE_PERIOD_IN_DAYS: i64 = 7;

#[utoipa::path(
    put,
    operation_id = "Edit artist",
    tag = "Artist",
    path = "/api/artists",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the artist", body=Artist),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<EditedArtist>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let approved_image_hosts = arc.settings.lock().unwrap().approved_image_hosts.clone();
    validate_image_urls(&form.pictures, &approved_image_hosts)?;

    let mut artist = arc.pool.find_artist_by_id(form.id).await?;

    // users can edit their own artist for a grace period of
    // 7 days after creation, to prevent e.g. hostile account takeovers.
    let has_permission = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditArtist)
        .await?;

    if !has_permission {
        let grace_period = chrono::Utc::now() - chrono::Duration::days(GRACE_PERIOD_IN_DAYS);
        if artist.created_by_id != user.sub || artist.created_at < grace_period {
            return Err(Error::InsufficientPermissions(format!(
                "{:?}",
                UserPermission::EditArtist
            )));
        }
    }

    if let Some(edits) = compute_diff(&artist, &form, &["id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "artist".to_string(),
                item_id: artist.id,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
        artist = arc.pool.update_artist_data(&form).await?;
    }

    Ok(HttpResponse::Ok().json(artist))
}
