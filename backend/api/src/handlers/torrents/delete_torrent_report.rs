use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpRequest, HttpResponse, web::Data, web::Query};
use arcadia_common::error::Result;
use arcadia_storage::models::torrent_report::DeleteTorrentReportQuery;
use arcadia_storage::models::user::UserPermission;
use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
    delete,
    operation_id = "Delete torrent report",
    tag = "Torrent",
    path = "/api/torrents/reports",
    security(
        ("http" = ["Bearer"])
    ),
    params(DeleteTorrentReportQuery),
    responses(
        (status = 200, description = "Successfully deleted the torrent report"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<DeleteTorrentReportQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteTorrentReport, req.path())
        .await?;

    arc.pool
        .delete_torrent_report(query.torrent_report_id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
