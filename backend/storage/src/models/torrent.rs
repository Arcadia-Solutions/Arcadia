use actix_multipart::form::{bytes::Bytes, text::Text, MultipartForm};
use arcadia_shared::tracker::models::torrent::InfoHash;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{
    edition_group::Source,
    title_group::{ContentType, TitleGroupCategory},
};
use sqlx::{prelude::FromRow, types::Json};
use strum::{Display, EnumString};
use utoipa::{IntoParams, ToSchema};

use crate::models::common::OrderByDirection;

use super::{torrent_report::TorrentReport, user::UserLite};

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "audio_codec_enum", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AudioCodec {
    Mp2,
    Mp3,
    Aac,
    Ac3,
    Dts,
    Flac,
    Pcm,
    #[sqlx(rename = "true-hd")]
    #[serde(rename = "true-hd")]
    TrueHd,
    Opus,
    Dsd,
    Cook,
    Wma,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "audio_channels_enum")]
pub enum AudioChannels {
    #[sqlx(rename = "1.0")]
    #[serde(rename = "1.0")]
    OneDotZero,
    #[sqlx(rename = "2.0")]
    #[serde(rename = "2.0")]
    TwoDotZero,
    #[sqlx(rename = "2.1")]
    #[serde(rename = "2.1")]
    TwoDotOne,
    #[sqlx(rename = "5.0")]
    #[serde(rename = "5.0")]
    FiveDotZero,
    #[sqlx(rename = "5.1")]
    #[serde(rename = "5.1")]
    FiveDotOne,
    #[sqlx(rename = "7.1")]
    #[serde(rename = "7.1")]
    SevenDotOne,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "audio_bitrate_sampling_enum")]
pub enum AudioBitrateSampling {
    #[sqlx(rename = "64")]
    #[serde(rename = "64")]
    Bitrate64,
    #[sqlx(rename = "128")]
    #[serde(rename = "128")]
    Bitrate128,
    #[sqlx(rename = "192")]
    #[serde(rename = "192")]
    Bitrate192,
    #[sqlx(rename = "256")]
    #[serde(rename = "256")]
    Bitrate256,
    #[sqlx(rename = "320")]
    #[serde(rename = "320")]
    Bitrate320,
    #[sqlx(rename = "APS (VBR)")]
    #[serde(rename = "APS (VBR)")]
    ApsVbr,
    #[sqlx(rename = "V2 (VBR)")]
    #[serde(rename = "V2 (VBR)")]
    V2Vbr,
    #[sqlx(rename = "V1 (VBR)")]
    #[serde(rename = "V1 (VBR)")]
    V1Vbr,
    #[sqlx(rename = "APX (VBR)")]
    #[serde(rename = "APX (VBR)")]
    ApxVbr,
    #[sqlx(rename = "V0 (VBR)")]
    #[serde(rename = "V0 (VBR)")]
    V0Vbr,
    Lossless,
    #[sqlx(rename = "24bit Lossless")]
    #[serde(rename = "24bit Lossless")]
    Lossless24Bit,
    #[sqlx(rename = "DSD64")]
    #[serde(rename = "DSD64")]
    Dsd64,
    #[sqlx(rename = "DSD128")]
    #[serde(rename = "DSD128")]
    Dsd128,
    #[sqlx(rename = "DSD256")]
    #[serde(rename = "DSD256")]
    Dsd256,
    #[sqlx(rename = "DSD512")]
    #[serde(rename = "DSD512")]
    Dsd512,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "video_codec_enum")]
pub enum VideoCodec {
    #[sqlx(rename = "mpeg1")]
    #[serde(rename = "mpeg1")]
    Mpeg1,
    #[sqlx(rename = "mpeg2")]
    #[serde(rename = "mpeg2")]
    Mpeg2,
    XviD,
    DivX,
    #[sqlx(rename = "h264")]
    #[serde(rename = "h264")]
    H264,
    #[sqlx(rename = "h265")]
    #[serde(rename = "h265")]
    H265,
    #[sqlx(rename = "vc-1")]
    #[serde(rename = "vc-1")]
    Vc1,
    #[sqlx(rename = "vp9")]
    #[serde(rename = "vp9")]
    Vp9,
    #[sqlx(rename = "av1")]
    #[serde(rename = "av1")]
    Av1,
    BD50,
    UHD100,
    #[sqlx(rename = "DVD5")]
    #[serde(rename = "DVD5")]
    Dvd5,
    #[sqlx(rename = "DVD9")]
    #[serde(rename = "DVD9")]
    Dvd9,
    #[sqlx(rename = "VP6")]
    #[serde(rename = "VP6")]
    Vp6,
    #[sqlx(rename = "RV40")]
    #[serde(rename = "RV40")]
    Rv40,
    #[sqlx(rename = "WMV2")]
    #[serde(rename = "WMV2")]
    Wmv2,
    #[sqlx(rename = "WMV1")]
    #[serde(rename = "WMV1")]
    Wmv1,
}

#[derive(Debug, Deserialize, Serialize, sqlx::Type, ToSchema, EnumString, Clone)]
#[sqlx(type_name = "language_enum")]
pub enum Language {
    Albanian,
    Arabic,
    Belarusian,
    Bengali,
    Bosnian,
    Bulgarian,
    Cantonese,
    Catalan,
    Chinese,
    #[sqlx(rename = "Chinese Simplified")]
    #[serde(rename = "Chinese Simplified")]
    #[strum(serialize = "Chinese Simplified")]
    ChineseSimplified,
    #[sqlx(rename = "Chinese Traditional")]
    #[serde(rename = "Chinese Traditional")]
    #[strum(serialize = "Chinese Traditional")]
    ChineseTraditional,
    Croatian,
    Czech,
    Danish,
    Dutch,
    #[strum(serialize = "en")]
    English,
    Estonian,
    Finnish,
    #[strum(serialize = "fr")]
    French,
    German,
    Greek,
    Hebrew,
    Hindi,
    Hungarian,
    Icelandic,
    Indonesian,
    Italian,
    Japanese,
    Kannada,
    Korean,
    Latvian,
    Lithuanian,
    Macedonian,
    Malay,
    Malayalam,
    Mandarin,
    Nepali,
    Norwegian,
    Persian,
    Polish,
    Portuguese,
    Romanian,
    Russian,
    Serbian,
    Slovak,
    Slovenian,
    Spanish,
    Swedish,
    Tamil,
    Tagalog,
    Telugu,
    Thai,
    Turkish,
    Ukrainian,
    Vietnamese,
    Wolof,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "video_resolution_enum")]
pub enum VideoResolution {
    NTSC,
    PAL,
    Other,
    #[sqlx(rename = "360p")]
    #[serde(rename = "360p")]
    P360,
    #[sqlx(rename = "480p")]
    #[serde(rename = "480p")]
    P480,
    #[sqlx(rename = "480i")]
    #[serde(rename = "480i")]
    I480,
    #[sqlx(rename = "576i")]
    #[serde(rename = "576i")]
    I576,
    #[sqlx(rename = "576p")]
    #[serde(rename = "576p")]
    P576,
    #[sqlx(rename = "720p")]
    #[serde(rename = "720p")]
    P720,
    #[sqlx(rename = "1080p")]
    #[serde(rename = "1080p")]
    P1080,
    #[sqlx(rename = "1080i")]
    #[serde(rename = "1080i")]
    I1080,
    #[sqlx(rename = "1440p")]
    #[serde(rename = "1440p")]
    P1440,
    #[sqlx(rename = "2160p")]
    #[serde(rename = "2160p")]
    P2160,
    #[sqlx(rename = "4320p")]
    #[serde(rename = "4320p")]
    P4320,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, ToSchema, EnumString)]
#[sqlx(type_name = "features_enum")]
pub enum Features {
    #[sqlx(rename = "HDR")]
    #[serde(rename = "HDR")]
    #[strum(serialize = "HDR")]
    Hdr,
    #[sqlx(rename = "HDR 10")]
    #[serde(rename = "HDR 10")]
    #[strum(serialize = "HDR 10")]
    HdrTen,
    #[sqlx(rename = "HDR 10+")]
    #[serde(rename = "HDR 10+")]
    #[strum(serialize = "HDR 10+")]
    HdrTenPlus,
    #[sqlx(rename = "DV")]
    #[serde(rename = "DV")]
    #[strum(serialize = "DV")]
    Dv,
    Commentary,
    Remux,
    #[sqlx(rename = "3D")]
    #[serde(rename = "3D")]
    #[strum(serialize = "3D")]
    ThreeD,
    #[sqlx(rename = "OCR")]
    #[serde(rename = "OCR")]
    #[strum(serialize = "OCR")]
    Ocr,
    Cue,
}

#[derive(Debug, Clone, Deserialize, Serialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "extras_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Extras {
    Booklet,
    Manual,
    BehindTheScenes,
    DeletedScenes,
    Featurette,
    Trailer,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TorrentFileEntry {
    pub name: String,
    pub size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TorrentFileList {
    pub parent_folder: String,
    pub files: Vec<TorrentFileEntry>,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct Torrent {
    pub id: i32,
    pub info_hash: InfoHash,
    pub upload_factor: i16,
    pub download_factor: i16,
    pub seeders: i64,
    pub leechers: i64,
    pub times_completed: i32,
    pub grabbed: i64,
    pub edition_group_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Local>,
    pub created_by_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub deleted_at: Option<DateTime<Local>>,
    pub deleted_by_id: Option<i64>,
    pub extras: Vec<Extras>,
    pub release_name: Option<String>,
    pub release_group: Option<String>,
    pub description: Option<String>, // specific to the torrent
    #[schema(value_type = HashMap<String, i32>)]
    pub file_amount_per_type: Json<Value>, // (5 mp3, 1 log, 5 jpg, etc.)
    pub uploaded_as_anonymous: bool,
    pub upload_method: String,
    #[schema(value_type = TorrentFileList)]
    pub file_list: Json<Value>,
    pub mediainfo: Option<String>,
    pub trumpable: Option<String>, // description of why it is trumpable
    pub staff_checked: bool,
    pub languages: Vec<Language>, // (fallback to original language) (english, french, etc.)
    pub container: String, // container of the main file (ex: if mkv movie and srt subs, mkv is the main)
    pub size: i64,         // in bytes
    // ---- audio
    pub duration: Option<i32>, // in seconds
    pub audio_codec: Option<AudioCodec>,
    pub audio_bitrate: Option<i32>, // in kb/s
    pub audio_bitrate_sampling: Option<AudioBitrateSampling>,
    pub audio_channels: Option<AudioChannels>,
    // ---- audio
    // ---- video
    pub video_codec: Option<VideoCodec>,
    pub features: Vec<Features>,
    pub subtitle_languages: Vec<Language>,
    pub video_resolution: Option<VideoResolution>, // ---- video
    pub video_resolution_other_x: Option<i32>,
    pub video_resolution_other_y: Option<i32>,
    pub extra_text: Option<String>,
    pub bonus_points_snatch_cost: i64,
}

#[derive(Debug, MultipartForm, FromRow, ToSchema)]
pub struct UploadedTorrent {
    #[schema(value_type = String)]
    pub extras: Text<String>,
    #[schema(value_type = String)]
    pub release_name: Text<String>,
    #[schema(value_type = Option<String>)]
    pub release_group: Option<Text<String>>,
    #[schema(value_type = Option<String>)]
    pub description: Option<Text<String>>,
    #[schema(value_type = bool)]
    pub uploaded_as_anonymous: Text<bool>,
    #[schema(value_type = Option<String>)]
    pub mediainfo: Option<Text<String>>,
    #[schema(value_type = String, format = Binary, content_media_type = "application/octet-stream")]
    pub torrent_file: Bytes,
    #[schema(value_type = String)]
    pub languages: Text<String>,
    #[schema(value_type = String)]
    pub container: Text<String>,
    #[schema(value_type = i32)]
    pub edition_group_id: Text<i32>,
    #[schema(value_type = Option<i32>)]
    pub duration: Option<Text<i32>>,
    #[schema(value_type = Option<AudioCodec>)]
    pub audio_codec: Option<Text<AudioCodec>>,
    #[schema(value_type = Option<i32>)]
    pub audio_bitrate: Option<Text<i32>>,
    #[schema(value_type = Option<String>)]
    pub audio_channels: Option<Text<AudioChannels>>,
    #[schema(value_type = Option<AudioBitrateSampling>)]
    pub audio_bitrate_sampling: Option<Text<AudioBitrateSampling>>,
    #[schema(value_type = Option<VideoCodec>)]
    pub video_codec: Option<Text<VideoCodec>>,
    #[schema(value_type = String)]
    pub features: Text<String>,
    #[schema(value_type = String)]
    pub subtitle_languages: Text<String>,
    #[schema(value_type = Option<VideoResolution>)]
    pub video_resolution: Option<Text<VideoResolution>>,
    #[schema(value_type = Option<i32>)]
    pub video_resolution_other_x: Option<Text<i32>>,
    #[schema(value_type = Option<i32>)]
    pub video_resolution_other_y: Option<Text<i32>>,
    #[schema(value_type = Option<String>)]
    pub trumpable: Option<Text<String>>,
    #[schema(value_type = Option<String>)]
    pub extra_text: Option<Text<String>>,
    #[schema(value_type = i64)]
    pub bonus_points_snatch_cost: Text<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedTorrent {
    pub id: i32,
    pub edition_group_id: i32,
    pub extras: Vec<Extras>,
    pub release_name: Option<String>,
    pub release_group: Option<String>,
    pub description: Option<String>,
    pub uploaded_as_anonymous: bool,
    pub mediainfo: Option<String>,
    pub container: String,
    pub languages: Vec<Language>,
    pub duration: Option<i32>,
    pub audio_codec: Option<AudioCodec>,
    pub audio_bitrate: Option<i32>,
    pub audio_bitrate_sampling: Option<AudioBitrateSampling>,
    pub audio_channels: Option<AudioChannels>,
    pub video_codec: Option<VideoCodec>,
    pub features: Vec<Features>,
    pub subtitle_languages: Vec<Language>,
    pub video_resolution: Option<VideoResolution>,
    pub video_resolution_other_x: Option<i32>,
    pub video_resolution_other_y: Option<i32>,
    pub trumpable: Option<String>,
    pub extra_text: Option<String>,
    pub bonus_points_snatch_cost: i64,
}

/// Message sent to the uploader of a torrent when it is marked as trumpable after its upload.
#[derive(Debug)]
pub struct TrumpableNotification {
    pub sender_id: i32,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Display)]
pub enum TorrentSearchOrderByColumn {
    #[serde(rename = "torrent_created_at")]
    #[strum(serialize = "torrent_created_at")]
    TorrentCreatedAt,
    #[serde(rename = "torrent_size")]
    #[strum(serialize = "torrent_size")]
    TorrentSize,
    #[serde(rename = "torrent_snatched_at")]
    #[strum(serialize = "torrent_snatched_at")]
    TorrentSnatchedAt,
    #[serde(rename = "title_group_original_release_date")]
    #[strum(serialize = "title_group_original_release_date")]
    TitleGroupOriginalReleaseDate,
    #[serde(rename = "torrent_seeders")]
    #[strum(serialize = "torrent_seeders")]
    TorrentSeeders,
    #[serde(rename = "torrent_leechers")]
    #[strum(serialize = "torrent_leechers")]
    TorrentLeechers,
    #[serde(rename = "torrent_snatched")]
    #[strum(serialize = "torrent_snatched")]
    TorrentSnatched,
    #[serde(rename = "torrent_bonus_points_snatch_cost")]
    #[strum(serialize = "torrent_bonus_points_snatch_cost")]
    TorrentBonusPointsSnatchCost,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TorrentSearch {
    // title group fields
    pub title_group_name: Option<String>,
    #[serde(default)]
    pub title_group_content_type: Vec<ContentType>,
    #[serde(default)]
    pub title_group_category: Vec<TitleGroupCategory>,
    pub title_group_tags: Option<String>,
    pub title_group_include_empty_groups: bool,
    // edition group fields
    #[serde(default)]
    pub edition_group_source: Vec<Source>,
    // torrent fields
    #[serde(default)]
    pub torrent_video_resolution: Vec<VideoResolution>,
    #[serde(default)]
    pub torrent_language: Vec<Language>,
    pub torrent_reported: Option<bool>,
    pub torrent_staff_checked: Option<bool>,
    pub torrent_trumpable: Option<bool>,
    pub torrent_created_by_id: Option<i32>,
    pub torrent_snatched_by_id: Option<i32>,
    // link to other tables
    pub artist_id: Option<i64>,
    pub collage_id: Option<i32>,
    pub series_id: Option<i64>,
    pub user_id_bookmarks: Option<i64>,
    // pagination and ordering
    pub page: i64,
    pub page_size: i64,
    pub order_by_column: TorrentSearchOrderByColumn,
    pub order_by_direction: OrderByDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TorrentHierarchyLite {
    pub id: i32,
    pub upload_factor: i16,
    pub download_factor: i16,
    pub seeders: i64,
    pub leechers: i64,
    pub times_completed: i32,
    pub grabbed: i64,
    pub edition_group_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by: Option<UserLite>,
    /// `created_by` is only filled for the uploader themselves and for the users allowed to see
    /// the information hidden by the paranoia settings, this tells them the upload is anonymous.
    pub uploaded_as_anonymous: bool,
    pub extras: Vec<Extras>,
    pub release_name: Option<String>,
    pub release_group: Option<String>,
    pub trumpable: Option<String>,
    pub staff_checked: bool,
    pub languages: Vec<Language>,
    pub container: String,
    pub size: i64,
    pub duration: Option<i32>,
    pub audio_codec: Option<AudioCodec>,
    pub audio_bitrate: Option<i32>,
    pub audio_bitrate_sampling: Option<AudioBitrateSampling>,
    pub audio_channels: Option<AudioChannels>,
    pub video_codec: Option<VideoCodec>,
    pub features: Vec<Features>,
    pub subtitle_languages: Vec<Language>,
    pub video_resolution: Option<VideoResolution>,
    pub video_resolution_other_x: Option<i32>,
    pub video_resolution_other_y: Option<i32>,
    pub extra_text: Option<String>,
    #[schema(value_type = Vec<TorrentReport>)]
    pub reports: Json<Vec<TorrentReport>>,
    pub peer_status: Option<PeerStatus>,
    pub bonus_points_snatch_cost: i64,
    pub upload_method: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Display, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PeerStatus {
    Seeding,
    Leeching,
    Snatched,
    Grabbed,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TorrentHierarchy {
    pub id: i32,
    pub upload_factor: i16,
    pub download_factor: i16,
    pub seeders: i64,
    pub leechers: i64,
    /// Last time a peer was seen seeding this torrent. Only set when it has no seeders left.
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_seeded_at: Option<DateTime<Local>>,
    /// When the current user last sent a reseed request for this torrent, if one is still
    /// outstanding. Only set when the torrent has no seeders left.
    #[schema(value_type = Option<String>, format = DateTime)]
    pub reseed_request_sent_at: Option<DateTime<Local>>,
    pub times_completed: i32,
    pub grabbed: i64,
    pub edition_group_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Local>,
    pub created_by_id: Option<i32>,
    pub created_by: Option<UserLite>,
    pub extras: Vec<Extras>,
    pub release_name: Option<String>,
    pub release_group: Option<String>,
    pub description: Option<String>,
    #[schema(value_type = HashMap<String, i32>)]
    pub file_amount_per_type: Json<Value>,
    pub uploaded_as_anonymous: bool,
    #[schema(value_type = TorrentFileList)]
    pub file_list: Json<Value>,
    pub mediainfo: Option<String>,
    pub trumpable: Option<String>,
    pub staff_checked: bool,
    pub languages: Vec<Language>,
    pub container: String,
    pub size: i64,
    pub duration: Option<i32>,
    pub audio_codec: Option<AudioCodec>,
    pub audio_bitrate: Option<i32>,
    pub audio_bitrate_sampling: Option<AudioBitrateSampling>,
    pub audio_channels: Option<AudioChannels>,
    pub video_codec: Option<VideoCodec>,
    pub features: Vec<Features>,
    pub subtitle_languages: Vec<Language>,
    pub video_resolution: Option<VideoResolution>,
    pub video_resolution_other_x: Option<i32>,
    pub video_resolution_other_y: Option<i32>,
    pub extra_text: Option<String>,
    pub reports: Vec<TorrentReport>,
    pub peer_status: Option<PeerStatus>,
    pub bonus_points_snatch_cost: i64,
    pub upload_method: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "torrent_deletion_reason_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TorrentDeletionReason {
    Trumped,
    Duplicate,
    Other,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TorrentToDelete {
    pub id: i32,
    pub deletion_reason: TorrentDeletionReason,
    pub extra_information: Option<String>,
    pub replacement_torrent_id: Option<i32>,
}
