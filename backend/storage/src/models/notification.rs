use crate::models::torrent::TorrentDeletionReason;
use crate::models::user::UserLite;
use arcadia_shared::tracker::models::announce_error_update::AnnounceErrorCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationForumSubCategoryThread {
    pub id: i64,
    pub forum_thread_id: i64,
    pub forum_thread_name: String,
    pub forum_sub_category_id: i32,
    pub forum_sub_category_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub read_status: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationForumThreadPost {
    pub id: i64,
    pub forum_post_id: i64,
    pub forum_thread_id: i64,
    pub forum_thread_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub read_status: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationTitleGroupComment {
    pub id: i64,
    /// exactly one of the comment and the review is set
    pub title_group_comment_id: Option<i64>,
    pub title_group_review_id: Option<i64>,
    pub title_group_id: i32,
    pub title_group_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub read_status: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationTitleGroupTorrent {
    pub id: i64,
    pub torrent_id: i32,
    pub title_group_id: i32,
    pub title_group_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub read_status: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct NotificationReseedRequest {
    pub torrent_id: i32,
    pub title_group_id: i32,
    pub title_group_name: String,
    /// Every user who has requested a reseed for this torrent (one line per torrent).
    pub requested_by: Vec<UserLite>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    /// True once the recipient has marked every request for this torrent as read.
    pub read_status: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationArtistTitleGroup {
    pub id: i64,
    pub artist_id: i64,
    pub artist_name: String,
    pub title_group_id: i32,
    pub title_group_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub read_status: bool,
}
#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationCollage {
    pub id: i64,
    pub collage_id: i64,
    pub collage_name: String,
    pub title_group_id: i32,
    pub title_group_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub read_status: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationTorrentRequestComment {
    pub id: i64,
    pub torrent_request_comment_id: i64,
    pub torrent_request_id: i64,
    pub title_group_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub read_status: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationStaffPmMessage {
    pub id: i64,
    pub staff_pm_message_id: i64,
    pub staff_pm_id: i64,
    pub staff_pm_subject: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub read_status: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationTorrentDeletion {
    pub torrent_id: i32,
    pub title_group_name: String,
    pub deletion_reason: TorrentDeletionReason,
    pub extra_information: Option<String>,
    pub replacement_torrent_id: Option<i32>,
    #[schema(value_type = String, format = DateTime)]
    pub deleted_at: DateTime<Utc>,
    pub read_status: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct NotificationAnnounceError {
    pub error_code: AnnounceErrorCode,
    /// hex encoded
    pub info_hash: String,
    /// None when the info_hash is not known by the tracker
    pub torrent_id: Option<i32>,
    pub title_group_id: Option<i32>,
    pub title_group_name: Option<String>,
    pub occurrences: i64,
    #[schema(value_type = String, format = DateTime)]
    pub first_seen_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct NotificationCounts {
    pub announcements: i32,
    pub conversations: i32,
    pub forum_sub_category_threads: i32,
    pub forum_thread_posts: i32,
    pub title_group_comments: i32,
    pub title_group_torrents: i32,
    pub artist_title_groups: i32,
    pub collages: i32,
    pub staff_pm_messages: i32,
    pub torrent_request_comments: i32,
    pub torrent_deletions: i32,
    pub announce_errors: i32,
    pub reseed_requests: i32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Notifications {
    pub forum_sub_category_threads: Vec<NotificationForumSubCategoryThread>,
    pub forum_thread_posts: Vec<NotificationForumThreadPost>,
    pub title_group_comments: Vec<NotificationTitleGroupComment>,
    pub title_group_torrents: Vec<NotificationTitleGroupTorrent>,
    pub artist_title_groups: Vec<NotificationArtistTitleGroup>,
    pub collages: Vec<NotificationCollage>,
    pub torrent_request_comments: Vec<NotificationTorrentRequestComment>,
    pub staff_pm_messages: Vec<NotificationStaffPmMessage>,
    pub torrent_deletions: Vec<NotificationTorrentDeletion>,
    pub announce_errors: Vec<NotificationAnnounceError>,
    pub reseed_requests: Vec<NotificationReseedRequest>,
}
