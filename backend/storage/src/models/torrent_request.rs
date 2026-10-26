use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

use crate::models::common::OrderByDirection;

use crate::models::{
    artist::{AffiliatedArtistHierarchy, AffiliatedArtistLite},
    edition_group::Source,
    series::SeriesLite,
    title_group::{TitleGroup, TitleGroupLite},
    torrent::AudioChannels,
    torrent_request_comment::TorrentRequestCommentHierarchy,
    torrent_request_vote::{TorrentRequestVoteHierarchy, UserCreatedTorrentRequestVote},
    user::UserLite,
};

use super::torrent::{
    AudioBitrateSampling, AudioCodec, Features, Language, VideoCodec, VideoResolution,
};

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TorrentRequest {
    pub id: i64,
    pub title_group_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    pub created_by_id: i32,
    pub filled_by_user_id: Option<i32>,
    pub filled_by_torrent_id: Option<i32>,
    #[schema(value_type = String, format = DateTime)]
    pub filled_at: Option<DateTime<Utc>>,
    pub edition_name: Option<String>,
    pub source: Vec<Source>,
    pub release_group: Option<String>,
    pub description: Option<String>,
    pub languages: Vec<Language>,
    pub container: Vec<String>,
    // ---- audio
    pub audio_codec: Vec<AudioCodec>,
    pub audio_channels: Vec<AudioChannels>,
    pub audio_bitrate_sampling: Vec<AudioBitrateSampling>,
    // ---- audio
    // ---- video
    pub video_codec: Vec<VideoCodec>,
    pub features: Vec<Features>,
    pub subtitle_languages: Vec<Language>,
    pub video_resolution: Vec<VideoResolution>, // ---- video
    pub video_resolution_other_x: Option<i32>,
    pub video_resolution_other_y: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserCreatedTorrentRequest {
    pub title_group_id: i32,
    pub edition_name: Option<String>,
    pub release_group: Option<String>,
    pub description: Option<String>,
    pub languages: Vec<Language>,
    pub container: Vec<String>,
    pub source: Vec<Source>,
    pub initial_vote: UserCreatedTorrentRequestVote,
    pub audio_codec: Vec<AudioCodec>,
    pub audio_channels: Vec<AudioChannels>,
    pub audio_bitrate_sampling: Vec<AudioBitrateSampling>,
    pub video_codec: Vec<VideoCodec>,
    pub features: Vec<Features>,
    pub subtitle_languages: Vec<Language>,
    pub video_resolution: Vec<VideoResolution>,
    pub video_resolution_other_x: Option<i32>,
    pub video_resolution_other_y: Option<i32>,
}
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct EditedTorrentRequest {
    pub id: i64,
    pub title_group_id: i32,
    pub edition_name: Option<String>,
    pub release_group: Option<String>,
    pub description: Option<String>,
    pub languages: Vec<Language>,
    pub container: Vec<String>,
    pub source: Vec<Source>,
    pub initial_vote: UserCreatedTorrentRequestVote,
    pub audio_codec: Vec<AudioCodec>,
    pub audio_channels: Vec<AudioChannels>,
    pub audio_bitrate_sampling: Vec<AudioBitrateSampling>,
    pub video_codec: Vec<VideoCodec>,
    pub features: Vec<Features>,
    pub subtitle_languages: Vec<Language>,
    pub video_resolution: Vec<VideoResolution>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TorrentRequestToDelete {
    pub id: i64,
    /// Only taken into account when the deletion is done with the delete_torrent_request permission
    pub refund_bounty: bool,
    /// Optional message sent to the voters, only taken into account when the deletion is done
    /// with the delete_torrent_request permission
    pub message: Option<String>,
}

/// Who is deleting the torrent request, which decides what the deletion is allowed to do
#[derive(Debug, Clone, Copy)]
pub enum TorrentRequestDeletionKind<'a> {
    /// The author of the request, only allowed while nobody else voted on it. They never get their
    /// bounty back, and have nobody to send a message to
    Author,
    /// A user with the delete_torrent_request permission
    WithPermission {
        refund_bounty: bool,
        /// Sent to the voters along with the deletion notification
        message: Option<&'a str>,
    },
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TorrentRequestBounty {
    pub bonus_points: i64,
    pub upload: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TorrentRequestHierarchyLite {
    pub torrent_request: TorrentRequest,
    pub created_by: UserLite,
    pub filled_by: Option<UserLite>,
    pub user_votes_amount: i32,
    pub bounty: TorrentRequestBounty,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TorrentRequestWithTitleGroupLite {
    pub torrent_request: TorrentRequestHierarchyLite,
    pub title_group: TitleGroupLite,
    pub affiliated_artists: Vec<AffiliatedArtistLite>,
    pub series: Option<SeriesLite>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TorrentRequestFill {
    pub torrent_request_id: i64,
    pub torrent_id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TorrentRequestAndAssociatedData {
    pub torrent_request: TorrentRequest,
    pub title_group: TitleGroup,
    pub affiliated_artists: Vec<AffiliatedArtistHierarchy>,
    pub series: SeriesLite,
    pub votes: Vec<TorrentRequestVoteHierarchy>,
    pub comments: Vec<TorrentRequestCommentHierarchy>,
    pub filled_by_user: Option<UserLite>,
    pub is_subscribed_to_comments: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, strum::Display, Default)]
pub enum TorrentRequestSearchOrderBy {
    #[serde(rename = "upload")]
    #[strum(serialize = "upload")]
    Upload,
    #[serde(rename = "bonus_points")]
    #[strum(serialize = "bonus_points")]
    BonusPoints,
    #[serde(rename = "voters")]
    #[strum(serialize = "voters")]
    Voters,
    #[default]
    #[serde(rename = "created_at")]
    #[strum(serialize = "created_at")]
    CreatedAt,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, utoipa::IntoParams)]
pub struct SearchTorrentRequestsQuery {
    pub title_group_name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub order_by: TorrentRequestSearchOrderBy,
    pub order_by_direction: OrderByDirection,
    pub include_filled: bool,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
