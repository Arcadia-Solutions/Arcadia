use arcadia_common::error::{Error, Result};
pub use arcadia_shared::tracker::models::env::SnatchedTorrentBonusPointsTransferredTo;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "displayed_top_bar_stats_enum", rename_all = "snake_case")]
pub enum DisplayedTopBarStats {
    Uploaded,
    Downloaded,
    Ratio,
    Torrents,
    ForumPosts,
    Seeding,
    Leeching,
    SeedingSize,
    AverageSeedingTime,
    BonusPoints,
    FreeleechTokens,
    CurrentStreak,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "displayable_user_stats_enum", rename_all = "snake_case")]
pub enum DisplayableUserStats {
    Uploaded,
    RealUploaded,
    Downloaded,
    RealDownloaded,
    Ratio,
    TitleGroups,
    EditionGroups,
    Torrents,
    ForumPosts,
    ForumThreads,
    TitleGroupComments,
    RequestComments,
    ArtistComments,
    Seeding,
    Leeching,
    Snatched,
    SeedingSize,
    RequestsFilled,
    CollagesStarted,
    RequestsVoted,
    AverageSeedingTime,
    Invited,
    Invitations,
    BonusPoints,
    FreeleechTokens,
    CurrentStreak,
    HighestStreak,
    JoinedAt,
    LastSeen,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(
    type_name = "torrent_request_vote_currency_enum",
    rename_all = "snake_case"
)]
pub enum TorrentRequestVoteCurrency {
    Upload,
    BonusPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "available_shop_item_enum", rename_all = "snake_case")]
pub enum AvailableShopItem {
    UploadAmount,
    FreeleechTokens,
    UserClassPromotion,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ArcadiaSettings {
    pub user_class_name_on_signup: String,
    pub default_css_sheet_name: String,
    pub open_signups: bool,
    pub global_upload_factor: i16,
    pub global_download_factor: i16,
    /// Site-wide factor (percentage, 100 = unchanged) applied to the bonus points a snatcher is
    /// charged when snatching a torrent.
    pub global_snatch_bonus_points_cost_factor: i16,
    /// Site-wide factor (percentage, 100 = unchanged) applied to the bonus points the uploader or
    /// current seeders receive when a torrent is snatched.
    pub global_snatch_bonus_points_reward_factor: i16,
    pub logo_subtitle: Option<String>,
    pub approved_image_hosts: Vec<String>,
    pub upload_page_top_text: Option<String>,
    pub automated_message_on_signup: Option<String>,
    pub automated_message_on_signup_sender_id: Option<i32>,
    pub automated_message_on_signup_locked: Option<bool>,
    pub automated_message_on_signup_conversation_name: Option<String>,
    pub bonus_points_given_on_upload: i64,
    pub allow_uploader_set_torrent_bonus_points_cost: bool,
    pub default_torrent_bonus_points_cost: i64,
    pub torrent_bonus_points_cost_min: i64,
    pub torrent_bonus_points_cost_max: i64,
    pub shop_upload_base_price_per_gb: i64,
    pub shop_upload_discount_tiers: serde_json::Value,
    pub shop_freeleech_token_base_price: i64,
    pub shop_freeleech_token_discount_tiers: serde_json::Value,
    pub bonus_points_alias: String,
    pub bonus_points_decimal_places: i16,
    #[schema(value_type = Option<String>)]
    pub torrent_max_release_date_allowed: Option<NaiveDate>,
    pub snatched_torrent_bonus_points_transferred_to:
        Option<SnatchedTorrentBonusPointsTransferredTo>,
    pub displayed_top_bar_stats: Vec<DisplayedTopBarStats>,
    pub displayable_user_stats: Vec<DisplayableUserStats>,
    pub torrent_request_vote_currencies: Vec<TorrentRequestVoteCurrency>,
    pub available_shop_items: Vec<AvailableShopItem>,
    #[schema(value_type = Vec<BonusPointsEndpoint>)]
    pub bonus_points_per_endpoint: Json<Vec<BonusPointsEndpoint>>,
    /// When enabled, a torrent seeded from several clients by the same user is rewarded once per
    /// client instead of once per torrent.
    pub reward_bonus_points_per_seeding_client: bool,
    /// When disabled, a user who already has activity on a torrent is never charged its bonus
    /// points snatch cost again when snatching it anew.
    pub charge_bonus_points_on_resnatch: bool,
    pub default_user_uploaded_on_registration: i64,
    pub default_user_downloaded_on_registration: i64,
    pub default_user_bonus_points_on_registration: i64,
    pub default_user_freeleech_tokens_on_registration: i32,
    pub display_image_host_drag_and_drop: bool,
    pub inactive_user_ban_after_days: Option<i32>,
    pub irc_webchat_enabled: bool,
    pub irc_webchat_default_channels: Vec<String>,
    pub min_amount_tags_title_group: i32,
    pub reseed_requestable_after_hours: i32,
    pub custom_js_code: Option<String>,
    pub custom_footer: Option<String>,
    pub invitation_expiration_days: i32,
    /// Appended to the message automatically sent to an uploader when their torrent is
    /// marked as trumpable after upload. Leave empty to send the notification without it.
    pub automated_message_on_torrent_marked_trumpable: Option<String>,
    /// When enabled, creating new title groups, edition groups and torrents is rejected.
    pub uploads_disabled: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct BonusPointsEndpoint {
    pub method: HttpMethod,
    pub path_prefix: String,
    pub probability: i16,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PublicArcadiaSettings {
    pub open_signups: bool,
    pub global_upload_factor: i16,
    pub global_download_factor: i16,
    pub logo_subtitle: Option<String>,
    pub bonus_points_alias: String,
    pub bonus_points_decimal_places: i16,
    pub displayed_top_bar_stats: Vec<DisplayedTopBarStats>,
    pub displayable_user_stats: Vec<DisplayableUserStats>,
    pub torrent_request_vote_currencies: Vec<TorrentRequestVoteCurrency>,
    pub available_shop_items: Vec<AvailableShopItem>,
    pub emails_enabled: bool,
    pub irc_enabled: bool,
    pub display_image_host_drag_and_drop: bool,
    pub irc_webchat_enabled: bool,
    pub irc_webchat_default_channels: Vec<String>,
    pub min_amount_tags_title_group: i32,
    pub reseed_requestable_after_hours: i32,
    pub custom_js_code: Option<String>,
    pub custom_footer: Option<String>,
    pub uploads_disabled: bool,
    /// set when an advent calendar is published and currently running
    pub running_advent_calendar_button_text: Option<String>,
}

impl ArcadiaSettings {
    /// The snatch cost an uploader is allowed to set on their torrent, the same policy applies
    /// at upload and at edit time.
    pub fn uploader_bonus_points_snatch_cost(&self, requested_cost: i64) -> Result<i64> {
        if !self.allow_uploader_set_torrent_bonus_points_cost {
            return Ok(self.default_torrent_bonus_points_cost);
        }
        if requested_cost < self.torrent_bonus_points_cost_min {
            return Err(Error::BonusPointsSnatchCostOutOfRange(format!(
                "bonus_points_snatch_cost must be at least {}",
                self.torrent_bonus_points_cost_min
            )));
        }
        if requested_cost > self.torrent_bonus_points_cost_max {
            return Err(Error::BonusPointsSnatchCostOutOfRange(format!(
                "bonus_points_snatch_cost must be at most {}",
                self.torrent_bonus_points_cost_max
            )));
        }
        Ok(requested_cost)
    }
}
