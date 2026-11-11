use actix_web::{HttpRequest, HttpResponse, web::Data};

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group::TitleGroupHierarchyLite,
        torrent::TorrentSearch,
        user::{HideableUserList, ParanoiaHiddenInformation, UserPermission},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search torrents",
    tag = "Search",
    params (TorrentSearch),
    path = "/api/search/torrents/lite",
    responses(
        (status = 200, description = "Title groups and their torrents found", body=PaginatedResults<TitleGroupHierarchyLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    req: HttpRequest,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let qs_config = serde_qs::Config::new().max_depth(5);
    let form: TorrentSearch = qs_config
        .deserialize_str(req.query_string())
        .map_err(|e| Error::InvalidTorrentSearchQuery(e.to_string()))?;

    if form.title_group_minimum_user_rating > form.title_group_maximum_user_rating {
        return Err(Error::InvalidTorrentSearchQuery(
            "the minimum user rating must not exceed the maximum".to_string(),
        ));
    }

    if let Some(user_id_bookmarks) = form.user_id_bookmarks
        && user_id_bookmarks != user.sub as i64
    {
        // later on we can add the possibility to view other users' bookmarks
        // protected by a user permission
        return Err(Error::InsufficientPermissions(
            "you can only view your own bookmarks".to_string(),
        ));
    }

    // a user searching torrents uploaded or snatched by someone else is asking for a list that
    // the paranoia settings of that user can hide
    let searched_lists: Vec<(i32, HideableUserList)> = [
        (form.torrent_created_by_id, HideableUserList::Torrents),
        (form.torrent_snatched_by_id, HideableUserList::Snatched),
    ]
    .into_iter()
    .filter_map(|(searched_user_id, list)| {
        searched_user_id
            .filter(|searched_user_id| *searched_user_id != user.sub)
            .map(|searched_user_id| (searched_user_id, list))
    })
    .collect();

    // the permission only matters when the search targets the lists of another user
    let can_see_paranoia_hidden_user_info = !searched_lists.is_empty()
        && arc
            .pool
            .user_has_permission(user.sub, &UserPermission::SeeParanoiaHiddenUserInfo)
            .await?;

    if !can_see_paranoia_hidden_user_info {
        for (searched_user_id, list) in searched_lists {
            if arc
                .pool
                .find_user_paranoia_settings(searched_user_id)
                .await?
                .is_list_hidden(list)
            {
                return Err(Error::InsufficientPermissions(
                    "this user hid this with their paranoia settings".to_string(),
                ));
            }
        }
    }

    let search_results = arc
        .pool
        .search_torrents(&form, Some(user.sub), can_see_paranoia_hidden_user_info)
        .await?;

    Ok(HttpResponse::Ok().json(search_results))
}
