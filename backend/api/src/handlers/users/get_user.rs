use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::OrderByDirection,
        torrent::{TorrentSearch, TorrentSearchOrderByColumn},
        user::{HideableUserList, ParanoiaHiddenInformation, PublicProfile, UserPermission},
    },
    redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetUserQuery {
    id: i32,
}

#[utoipa::path(
    get,
    operation_id = "Get user",
    tag = "User",
    path = "/api/users",
    params(GetUserQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the user's profile", body=PublicProfile),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<GetUserQuery>,
    requesting_user: Authdata,
) -> Result<HttpResponse> {
    let mut user = arc.pool.find_user_profile(&query.id).await?;

    let can_see_paranoia_hidden_user_info = requesting_user.sub == query.id
        || arc
            .pool
            .user_has_permission(
                requesting_user.sub,
                &UserPermission::SeeParanoiaHiddenUserInfo,
            )
            .await?;
    let show_uploaded_torrents =
        can_see_paranoia_hidden_user_info || !user.is_list_hidden(HideableUserList::Torrents);
    let show_snatched_torrents =
        can_see_paranoia_hidden_user_info || !user.is_list_hidden(HideableUserList::Snatched);
    if !can_see_paranoia_hidden_user_info {
        user.hide_paranoia_hidden_stats();
    }

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
        torrent_created_by_id: Some(query.id),
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
    let uploaded_torrents = if show_uploaded_torrents {
        arc.pool
            .search_torrents(
                &torrent_search,
                Some(requesting_user.sub),
                can_see_paranoia_hidden_user_info,
            )
            .await?
            .results
    } else {
        vec![]
    };
    torrent_search.torrent_snatched_by_id = Some(query.id);
    torrent_search.torrent_created_by_id = None;
    torrent_search.order_by_column = TorrentSearchOrderByColumn::TorrentSnatchedAt;
    let snatched_torrents = if show_snatched_torrents {
        arc.pool
            .search_torrents(
                &torrent_search,
                Some(requesting_user.sub),
                can_see_paranoia_hidden_user_info,
            )
            .await?
            .results
    } else {
        vec![]
    };

    let can_see_torrent_clients = arc
        .pool
        .user_has_permission(
            requesting_user.sub,
            &UserPermission::SeeForeignTorrentClients,
        )
        .await?;

    let torrent_clients = if can_see_torrent_clients {
        arc.pool.get_user_torrent_clients(query.id).await?
    } else {
        vec![]
    };

    let earned_badges = arc.pool.find_user_earned_badges(query.id).await?;

    Ok(HttpResponse::Ok().json(PublicProfile {
        user,
        last_five_uploaded_torrents: uploaded_torrents,
        last_five_snatched_torrents: snatched_torrents,
        torrent_clients,
        earned_badges,
    }))
}
