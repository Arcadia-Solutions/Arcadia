use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use utoipa::{IntoParams, ToSchema};

use crate::models::{
    common::{OrderByDirection, PaginatedResults},
    user::UserLiteAvatar,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(
    type_name = "advent_calendar_late_opening_policy_enum",
    rename_all = "snake_case"
)]
pub enum AdventCalendarLateOpeningPolicy {
    DuringDoorWindow,
    GracePeriod,
    UntilEventEnd,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(
    type_name = "advent_calendar_reward_distribution_enum",
    rename_all = "snake_case"
)]
pub enum AdventCalendarRewardDistribution {
    Linear,
    Gaussian,
    Logarithmic,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdventCalendar {
    pub id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by_id: i32,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub starts_at: DateTime<Utc>,
    pub top_message: String,
    pub button_text: String,
    pub staff_notes: String,
    pub late_opening_policy: AdventCalendarLateOpeningPolicy,
    pub late_opening_grace_hours: i32,
    pub published: bool,
    #[schema(value_type = Vec<AdventCalendarDoor>)]
    pub doors: Json<Vec<AdventCalendarDoor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdventCalendarLite {
    pub id: i32,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub starts_at: DateTime<Utc>,
    pub published: bool,
    pub door_count: i64,
    pub total_duration_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdventCalendarDoor {
    pub id: i32,
    pub position: i32,
    #[serde(flatten)]
    #[schema(inline)]
    pub settings: AdventCalendarDoorSettings,
}

/// Used to create and to edit a calendar, the doors' positions are their index in the list
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedAdventCalendar {
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub starts_at: DateTime<Utc>,
    pub top_message: String,
    pub button_text: String,
    pub staff_notes: String,
    pub late_opening_policy: AdventCalendarLateOpeningPolicy,
    pub late_opening_grace_hours: i32,
    pub published: bool,
    pub doors: Vec<UserCreatedAdventCalendarDoor>,
}

/// Doors without an id are created, doors with an id are updated
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedAdventCalendarDoor {
    pub id: Option<i32>,
    #[serde(flatten)]
    #[schema(inline)]
    pub settings: AdventCalendarDoorSettings,
}

/// The fields of a door that staff can edit
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdventCalendarDoorSettings {
    pub name: String,
    pub image_url: Option<String>,
    pub duration_minutes: i32,
    pub opening_message: String,
    /// shown above the passphrase input while the door can be opened
    pub locked_message: Option<String>,
    pub passphrase: Option<String>,
    pub staff_notes: String,
    pub upload_min: i64,
    pub upload_max: i64,
    pub upload_distribution: AdventCalendarRewardDistribution,
    pub download_removed_min: i64,
    pub download_removed_max: i64,
    pub download_removed_distribution: AdventCalendarRewardDistribution,
    pub freeleech_tokens_min: i64,
    pub freeleech_tokens_max: i64,
    pub freeleech_tokens_distribution: AdventCalendarRewardDistribution,
    pub invitations_min: i64,
    pub invitations_max: i64,
    pub invitations_distribution: AdventCalendarRewardDistribution,
    pub bonus_points_min: i64,
    pub bonus_points_max: i64,
    pub bonus_points_distribution: AdventCalendarRewardDistribution,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdventCalendarDoorState {
    Locked,
    Openable,
    Opened,
    Missed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct AdventCalendarDoorOpeningResult {
    pub opening_message: String,
    pub upload: i64,
    pub download_removed: i64,
    pub freeleech_tokens: i32,
    pub invitations: i32,
    pub bonus_points: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdventCalendarDoorForUser {
    pub id: i32,
    pub position: i32,
    pub name: String,
    pub image_url: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub starts_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub ends_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub openable_until: DateTime<Utc>,
    pub requires_passphrase: bool,
    pub locked_message: Option<String>,
    pub state: AdventCalendarDoorState,
    pub opening: Option<AdventCalendarDoorOpeningResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublishedAdventCalendar {
    pub id: i32,
    pub name: String,
    pub top_message: String,
    #[schema(value_type = String, format = DateTime)]
    pub starts_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub ends_at: DateTime<Utc>,
    pub doors: Vec<AdventCalendarDoorForUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdventCalendarDoorOpeningAttempt {
    pub passphrase: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AdventCalendarDoorOpening {
    pub door_id: i32,
    pub upload: i64,
    pub download_removed: i64,
    pub freeleech_tokens: i32,
    pub invitations: i32,
    pub bonus_points: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, strum::Display)]
pub enum AdventCalendarDoorOpenerOrderByColumn {
    #[serde(rename = "opened_at")]
    #[strum(serialize = "opened_at")]
    OpenedAt,
    #[serde(rename = "username")]
    #[strum(serialize = "username")]
    Username,
    #[serde(rename = "upload")]
    #[strum(serialize = "upload")]
    Upload,
    #[serde(rename = "download_removed")]
    #[strum(serialize = "download_removed")]
    DownloadRemoved,
    #[serde(rename = "freeleech_tokens")]
    #[strum(serialize = "freeleech_tokens")]
    FreeleechTokens,
    #[serde(rename = "invitations")]
    #[strum(serialize = "invitations")]
    Invitations,
    #[serde(rename = "bonus_points")]
    #[strum(serialize = "bonus_points")]
    BonusPoints,
}

/// `username` is a partial, case insensitive match on the users who opened the door (empty
/// matches every user)
#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AdventCalendarDoorStatsQuery {
    pub username: String,
    pub order_by_column: AdventCalendarDoorOpenerOrderByColumn,
    pub order_by_direction: OrderByDirection,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdventCalendarDoorOpener {
    pub user: UserLiteAvatar,
    #[schema(value_type = String, format = DateTime)]
    pub opened_at: DateTime<Utc>,
    pub upload: i64,
    pub download_removed: i64,
    pub freeleech_tokens: i32,
    pub invitations: i32,
    pub bonus_points: i64,
}

/// sums of the rewards granted by every opening of a door
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdventCalendarDoorRewardTotals {
    pub upload: i64,
    pub download_removed: i64,
    pub freeleech_tokens: i64,
    pub invitations: i64,
    pub bonus_points: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdventCalendarDoorStats {
    /// amount of users who opened the door, regardless of the username filter
    pub opening_count: i64,
    pub reward_totals: AdventCalendarDoorRewardTotals,
    pub openers: PaginatedResults<AdventCalendarDoorOpener>,
}
