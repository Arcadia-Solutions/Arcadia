use crate::Arcadia;
use actix_web::HttpResponse;
use actix_web::web::{Data, Query};
use arcadia_common::error::Result;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::torrent_request::{
    SearchTorrentRequestsQuery, TorrentRequestWithTitleGroupLite,
};

use arcadia_storage::redis::RedisPoolInterface;

#[utoipa::path(
    get,
    operation_id = "Search torrent requests",
    tag = "Search",
    path = "/api/search/torrent-requests",
    params(SearchTorrentRequestsQuery),
    responses(
        (status = 200, description = "Paginated list of torrent requests with associated title groups", body = PaginatedResults<TorrentRequestWithTitleGroupLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<SearchTorrentRequestsQuery>,
) -> Result<HttpResponse> {
    let results = arc.pool.search_torrent_requests(&query, None).await?;
    Ok(HttpResponse::Ok().json(results))
}
