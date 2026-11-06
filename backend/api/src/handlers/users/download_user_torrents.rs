use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{
    Arcadia, handlers::torrents::download_torrents_archive::stream_torrents_archive,
    middlewares::auth_middleware::Authdata,
};
use arcadia_common::error::Result;

#[derive(Debug, Deserialize, Clone, Copy, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserTorrentsArchiveKind {
    Uploaded,
    Snatched,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct DownloadUserTorrentsQuery {
    /// Which set of the current user's torrents to archive.
    #[serde(rename = "type")]
    pub kind: UserTorrentsArchiveKind,
}

#[utoipa::path(
    get,
    operation_id = "Download the current user's torrents as a zip archive",
    tag = "User",
    path = "/api/users/me/torrents-archive",
    params (DownloadUserTorrentsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully built the zip archive of the user's torrents", content_type = "application/zip"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<DownloadUserTorrentsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DownloadTorrent, req.path())
        .await?;

    // Always the current user's own torrents: there is no target-user parameter, so no one can
    // download someone else's torrents through this endpoint.
    let (torrent_ids, archive_label) = match query.kind {
        UserTorrentsArchiveKind::Uploaded => (
            arc.pool.get_uploaded_torrent_ids(user.sub).await?,
            "uploaded",
        ),
        UserTorrentsArchiveKind::Snatched => (
            arc.pool.get_snatched_torrent_ids(user.sub).await?,
            "snatched",
        ),
    };

    let archive_file_name = format!("{} {archive_label} torrents.zip", arc.tracker.name);

    stream_torrents_archive(arc.clone(), user.sub, torrent_ids, archive_file_name).await
}
