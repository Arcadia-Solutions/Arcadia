use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};

use chrono::Local;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{torrent::Torrent, user::UserPermission, user_edit_change_log::NewUserEditChangeLog},
    redis::RedisPoolInterface,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MoveTorrentToEditionGroup {
    pub torrent_id: i32,
    pub target_edition_group_id: i32,
}

#[utoipa::path(
    put,
    operation_id = "Move torrent to edition group",
    tag = "Torrent",
    path = "/api/torrents/move-to-edition-group",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully moved the torrent to the target edition group", body=Torrent),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<MoveTorrentToEditionGroup>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let has_permission = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::MoveTorrentToOtherEditionGroup)
        .await?;

    let torrent = arc.pool.find_torrent(form.torrent_id).await?;

    if !has_permission {
        if torrent.created_by_id != user.sub {
            arc.pool
                .require_permission(
                    user.sub,
                    &UserPermission::MoveTorrentToOtherEditionGroup,
                    req.path(),
                )
                .await?;
        }

        let hours_since_upload = (Local::now() - torrent.created_at).num_hours();
        if hours_since_upload >= 24 {
            return Err(Error::TorrentMoveWindowExpired);
        }
    }

    let source_edition_group = arc
        .pool
        .find_edition_group(torrent.edition_group_id)
        .await?;
    let target_edition_group = arc
        .pool
        .find_edition_group(form.target_edition_group_id)
        .await?;

    if source_edition_group.title_group_id != target_edition_group.title_group_id {
        return Err(Error::EditionGroupsNotInSameTitleGroup);
    }

    arc.pool
        .create_user_edit_change_log(&NewUserEditChangeLog {
            item_type: "torrent".to_string(),
            item_id: torrent.id as i64,
            edited_by_id: user.sub,
            edits: serde_json::json!({
                "edition_group_id": {
                    "old": torrent.edition_group_id,
                    "new": form.target_edition_group_id
                }
            }),
        })
        .await?;

    arc.pool
        .move_torrent_to_edition_group(form.torrent_id, form.target_edition_group_id)
        .await?;

    let updated_torrent = arc.pool.find_torrent(form.torrent_id).await?;
    Ok(HttpResponse::Ok().json(updated_torrent))
}
