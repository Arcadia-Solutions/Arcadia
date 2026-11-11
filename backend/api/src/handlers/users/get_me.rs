use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::OrderByDirection,
        torrent::{TorrentSearch, TorrentSearchOrderByColumn},
        user::Profile,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get me",
    tag = "User",
    path = "/api/users/me",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the user's profile", body=Profile),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let mut current_user = arc.pool.find_user_with_id(user.sub).await?;
    current_user.password_hash = String::from("");
    let torrent_clients = arc.pool.get_user_torrent_clients(current_user.id).await?;
    let user_warnings = arc.pool.find_user_warnings(current_user.id).await;

    let mut torrent_search = TorrentSearch {
        title_group_name: None,
        title_group_tags: None,
        title_group_include_empty_groups: false,
        title_group_content_type: Vec::new(),
        title_group_category: Vec::new(),
        edition_group_source: Vec::new(),
        torrent_video_resolution: Vec::new(),
        torrent_language: Vec::new(),
        torrent_reported: None,
        torrent_staff_checked: None,
        torrent_trumpable: None,
        torrent_created_by_id: Some(current_user.id),
        torrent_snatched_by_id: None,
        page: 1,
        page_size: 5,
        order_by_column: TorrentSearchOrderByColumn::TorrentCreatedAt,
        order_by_direction: OrderByDirection::Desc,
        artist_id: None,
        collage_id: None,
        series_id: None,
        user_id_bookmarks: None,
    };
    let uploaded_torrents = arc
        .pool
        .search_torrents(&torrent_search, Some(current_user.id), false)
        .await?;
    torrent_search.torrent_snatched_by_id = Some(current_user.id);
    torrent_search.torrent_created_by_id = None;
    torrent_search.order_by_column = TorrentSearchOrderByColumn::TorrentSnatchedAt;
    let snatched_torrents = arc
        .pool
        .search_torrents(&torrent_search, Some(current_user.id), false)
        .await?;

    let earned_badges = arc
        .pool
        .find_user_earned_badges(current_user.id, Some(super::PROFILE_EARNED_BADGES_AMOUNT))
        .await?;

    Ok(HttpResponse::Ok().json(Profile {
        user: current_user,
        torrent_clients,
        user_warnings,
        last_five_uploaded_torrents: uploaded_torrents.results,
        last_five_snatched_torrents: snatched_torrents.results,
        earned_badges,
    }))
}
