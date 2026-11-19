use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct HomeStats {
    pub enabled_users: i64,
    pub users_active_today: i64,
    pub users_active_this_week: i64,
    pub users_active_this_month: i64,
    pub torrents: i64,
    pub titles: i64,
    pub artists: i64,
    pub peers: i64,
    pub seeders: i64,
    pub leechers: i64,
    pub torrent_requests: i64,
    pub torrent_requests_filled: i64,
    pub snatches: i64,
    pub series: i64,
    pub collages: i64,
    pub forum_threads: i64,
    pub forum_posts: i64,
    pub title_group_comments: i64,
    pub title_group_ratings: i64,
    pub torrent_request_comments: i64,
}
