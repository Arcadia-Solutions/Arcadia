use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::subscription::SearchSubscriptionsQuery;
use arcadia_storage::models::torrent_request::TorrentRequestWithTitleGroupLite;
use arcadia_storage::redis::RedisPoolInterface;

use arcadia_storage::models::torrent_request::{
    SearchTorrentRequestsQuery, TorrentRequestSearchOrderBy,
};

#[utoipa::path(
    get,
    operation_id = "Get torrent request comments subscriptions",
    tag = "Subscription",
    path = "/api/subscriptions/torrent-request-comments",
    params(SearchSubscriptionsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully retrieved torrent request comments subscriptions", body = PaginatedResults<TorrentRequestWithTitleGroupLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchSubscriptionsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let torrent_request_search = SearchTorrentRequestsQuery {
        title_group_name: None,
        tags: None,
        order_by: TorrentRequestSearchOrderBy::CreatedAt,
        order_by_direction: query.order_by_direction,
        include_filled: true,
        page: Some(query.page as i64),
        page_size: Some(query.page_size as i64),
    };

    let results = arc
        .pool
        .search_torrent_requests(&torrent_request_search, Some(user.sub))
        .await?;

    Ok(HttpResponse::Ok().json(results))
}
