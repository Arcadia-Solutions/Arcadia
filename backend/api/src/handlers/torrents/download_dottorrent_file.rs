use actix_web::{
    HttpRequest, HttpResponse,
    http::header::{
        Charset, ContentDisposition, ContentType, DispositionParam, DispositionType, ExtendedValue,
    },
    web::{Data, Query},
};
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::Result;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct DownloadTorrentQuery {
    id: i32,
}

#[utoipa::path(
    get,
    operation_id = "Download torrent file",
    tag = "Torrent",
    path = "/api/torrents",
    params (DownloadTorrentQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully downloaded the torrent file"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<DownloadTorrentQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DownloadTorrent, req.path())
        .await?;

    let torrent = arc
        .pool
        .get_torrent(
            user.sub,
            query.id,
            &arc.tracker.name,
            arc.api.frontend_url.as_ref(),
            arc.tracker.url.as_ref(),
            arc.tracker.torrent_source_tag.as_deref(),
        )
        .await?;

    let cd = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![
            DispositionParam::FilenameExt(ExtendedValue {
                charset: Charset::Ext(String::from("UTF-8")),
                language_tag: None,
                value: format!("{}.torrent", torrent.title).into_bytes(),
            }),
            // TODO: add fallback for better compatibility
            //DispositionParam::Filename(format!("{}.torrent", name_ascii))
        ],
    };

    Ok(HttpResponse::Ok()
        .insert_header(ContentType::octet_stream())
        .insert_header(cd)
        .body(torrent.file_contents))
}
