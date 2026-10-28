use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_storage::utils::compute_diff;

use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        torrent::{EditedTorrent, Torrent, TrumpableNotification},
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit torrent",
    tag = "Torrent",
    path = "/api/torrents",
    description = "Changing the `trumpable` field requires the `edit_torrent_trumpable` permission (can otherwise be set at creation time by the uploader). Setting it sends a message to the uploader.",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the torrent", body=Torrent),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<EditedTorrent>,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
    user: Authdata,
) -> Result<HttpResponse> {
    let torrent = arc.pool.find_torrent(form.id).await?;

    let can_edit_any_torrent = arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditTorrent)
        .await?;
    if !can_edit_any_torrent && torrent.created_by_id != user.sub {
        return Err(Error::InsufficientPermissions(format!(
            "{:?}",
            UserPermission::EditTorrent
        )));
    }

    // the uploader is held to the same snatch cost policy as at upload time
    if !can_edit_any_torrent
        && form.bonus_points_snatch_cost != torrent.bonus_points_snatch_cost
        && arc
            .settings
            .lock()
            .unwrap()
            .uploader_bonus_points_snatch_cost(form.bonus_points_snatch_cost)?
            != form.bonus_points_snatch_cost
    {
        return Err(Error::BonusPointsSnatchCostOutOfRange(
            "the bonus_points_snatch_cost can not be set by uploaders".to_string(),
        ));
    }

    // the trumpable field can be set by the uploader at creation time,
    // editing it afterwards requires a dedicated permission.
    let trumpable_changed = torrent.trumpable != form.trumpable;
    if trumpable_changed {
        arc.pool
            .require_permission(user.sub, &UserPermission::EditTorrentTrumpable, req.path())
            .await?;
    }

    // a torrent newly marked as trumpable after its upload must be notified to its uploader,
    // editing an already set reason and removing the mark both need no notification.
    let trumpable_notification = if trumpable_changed
        && torrent.created_by_id != user.sub
        && let Some(trump_reason) = form.trumpable.as_deref()
    {
        let title_group_name = arc.pool.get_torrent_title_group_name(torrent.id).await?;
        let trumpable_notice = format!(
            "Your torrent [url=/torrent/{}]{title_group_name}[/url] has been marked as trumpable for the following reason: {trump_reason}.",
            torrent.id
        );

        let message_footer = arc
            .settings
            .lock()
            .unwrap()
            .automated_message_on_torrent_marked_trumpable
            .clone();

        let message = match message_footer
            .as_deref()
            .map(str::trim)
            .filter(|footer| !footer.is_empty())
        {
            Some(footer) => format!("{trumpable_notice}\n{footer}"),
            None => trumpable_notice,
        };

        Some(TrumpableNotification {
            sender_id: user.sub,
            content: message,
        })
    } else {
        None
    };

    if let Some(edits) = compute_diff(&torrent, &form, &["id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "torrent".to_string(),
                item_id: torrent.id as i64,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let updated_torrent = arc
        .pool
        .update_torrent(&form, torrent.id, trumpable_notification.as_ref())
        .await?;

    Ok(HttpResponse::Ok().json(updated_torrent))
}
