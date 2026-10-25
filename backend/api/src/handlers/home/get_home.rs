use crate::Arcadia;
use actix_web::{web::Data, HttpResponse};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        forum::{ForumPostAndThreadName, ForumSearchQuery, ForumSearchResult},
        home_stats::HomeStats,
        site_highlight::SiteHighlightForHome,
        title_group::TitleGroupLite,
        title_group_comment::TitleGroupCommentSearchResult,
    },
    redis::RedisPoolInterface,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HomePage {
    recent_announcements: Vec<ForumPostAndThreadName>,
    stats: HomeStats,
    latest_uploads: Vec<TitleGroupLite>,
    latest_posts_in_threads: Vec<ForumSearchResult>,
    latest_title_group_comments: Vec<TitleGroupCommentSearchResult>,
    bonus_points_alias: String,
    site_highlights: Vec<SiteHighlightForHome>,
}

#[utoipa::path(
    get,
    operation_id = "Get home data",
    tag = "Home",
    path = "/api/home",
    responses(
        (status = 200, description = "Successfully got the home page", body=HomePage),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(arc: Data<Arcadia<R>>) -> Result<HttpResponse> {
    let search_forum_threads_form = ForumSearchQuery {
        thread_name: None,
        page_size: 5,
        page: 1,
    };
    let title_group_filter = None;

    let (
        recent_announcements,
        stats,
        latest_uploads_in_title_groups,
        latest_posts_in_threads,
        latest_title_group_comments,
        site_highlights,
    ) = tokio::try_join!(
        arc.pool.find_first_thread_posts_in_sub_category(1, 5),
        arc.pool.find_home_stats(),
        arc.pool
            .find_title_group_info_lite(None, Some(""), &title_group_filter, 5),
        arc.pool.search_forum_threads(&search_forum_threads_form),
        arc.pool.find_latest_title_group_comments(5),
        arc.pool.find_enabled_site_highlights_for_home(),
    )?;

    let bonus_points_alias = arc.settings.lock().unwrap().bonus_points_alias.clone();

    Ok(HttpResponse::Created().json(json!({
        "recent_announcements":recent_announcements,
        "stats": stats,
        "latest_uploads": latest_uploads_in_title_groups,
        "latest_posts_in_threads": latest_posts_in_threads.results,
        "latest_title_group_comments": latest_title_group_comments,
        "bonus_points_alias": bonus_points_alias,
        "site_highlights": site_highlights,
    })))
}
