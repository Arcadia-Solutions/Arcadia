use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::Decode;
use utoipa::ToSchema;

use crate::models::arcadia_settings::DisplayableUserStats;
use crate::models::common::OrderByDirection;
use crate::models::peer::TorrentClient;

use super::title_group::TitleGroupHierarchyLite;
use super::user_badge::UserEarnedBadgeWithDetails;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
    pub email: String,
    pub password_hash: String,
    #[schema(value_type = String, format = "0.0.0.0")]
    pub registered_from_ip: IpNetwork,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub uploaded: i64,
    pub real_uploaded: i64,
    pub downloaded: i64,
    pub real_downloaded: i64,
    #[schema(value_type = String, format = DateTime)]
    pub last_seen: DateTime<Utc>,
    pub class_name: String,
    pub class_locked: bool,
    pub permissions: Vec<UserPermission>,
    pub title_groups: i32,
    pub edition_groups: i32,
    pub torrents: i32,
    pub forum_posts: i32,
    pub forum_threads: i32,
    pub title_group_comments: i32,
    pub request_comments: i32,
    pub artist_comments: i64,
    pub seeding: i32,
    pub leeching: i32,
    pub snatched: i32,
    pub seeding_size: i64,
    pub requests_filled: i64,
    pub collages_started: i64,
    pub requests_voted: i64,
    pub average_seeding_time: i64, //in seconds
    pub invited: i64,
    pub invitations: i16,
    pub bonus_points: i64,
    pub freeleech_tokens: i32,
    pub warned: bool,
    pub banned: bool,
    pub passkey: String,
    pub css_sheet_name: String,
    pub current_streak: i32,
    pub highest_streak: i32,
    pub custom_title: Option<String>,
    pub max_snatches_per_day: Option<i32>,
    pub irc_password: Option<String>,
    pub irc_site_embed_enabled: bool,
    pub show_upload_method: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "user_permissions_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserPermission {
    UploadTorrent,
    DownloadTorrent,
    CreateTorrentRequest,
    ImmuneActivityPruning,
    EditTitleGroup,
    EditTitleGroupComment,
    EditEditionGroup,
    EditTorrent,
    EditArtist,
    DeleteArtist,
    DeleteTitleGroup,
    EditCollage,
    DeleteCollage,
    EditSeries,
    DeleteSeries,
    RemoveTitleGroupFromSeries,
    EditTorrentRequest,
    EditForumPost,
    EditForumThread,
    PinForumThread,
    LockForumThread,
    EditForumSubCategory,
    EditForumCategory,
    CreateForumCategory,
    CreateForumSubCategory,
    CreateForumThread,
    CreateForumPost,
    DeleteForumCategory,
    DeleteForumSubCategory,
    DeleteForumThread,
    DeleteForumPost,
    DeleteTitleGroupComment,
    DeleteTorrentRequestComment,
    DeleteTorrentRequest,
    SendPm,
    CreateCssSheet,
    EditCssSheet,
    ReadStaffPm,
    ReplyStaffPm,
    ResolveStaffPm,
    UnresolveStaffPm,
    ManageTitleGroupTags,
    EditTitleGroupTag,
    DeleteTorrent,
    RequestReseed,
    SetTorrentStaffChecked,
    GetUserApplication,
    UpdateUserApplication,
    WarnUser,
    BanUser,
    RemoveUserWarning,
    EditUser,
    GenerateResetPasswordToken,
    CreateWikiArticle,
    EditWikiArticle,
    LinkSimilarWikiArticles,
    CreateUserClass,
    EditUserClass,
    DeleteUserClass,
    SetAndViewUserPermissions,
    LockUserClass,
    ChangeUserClass,
    EditArcadiaSettings,
    CreateDonation,
    EditDonation,
    DeleteDonation,
    SearchDonation,
    SearchUsers,
    SearchUnauthorizedAccess,
    SearchUserEditChangeLogs,
    DeleteUserEditChangeLog,
    ViewTorrentPeersAndActivities,
    EditTorrentUpDownFactors,
    DeleteCollageEntry,
    DeleteTorrentReport,
    SeeForeignTorrentClients,
    SetUserCustomTitle,
    MergeTitleGroup,
    DeleteEditionGroup,
    MoveTorrentToOtherEditionGroup,
    ViewStatsDetails,
    ReadAllConversations,
    CreateUserBadge,
    EditUserBadge,
    DeleteUserBadge,
    ViewInvisibleUserBadges,
    CreateUserBadgeCategory,
    EditUserBadgeCategory,
    DeleteUserBadgeCategory,
    AwardUserBadge,
    RevokeUserBadge,
    ManageSiteHighlights,
    ManageRelatedForumThread,
    CreateForumPollVote,
    UseMaintenanceTools,
    EditTorrentTrumpable,
    LinkSimilarTitleGroup,
    UnlinkSimilarTitleGroup,
    SendMassPm,
    SeeParanoiaHiddenUserInfo,
    SeeForeignBonusPointsLogs,
    ReactToContent,
    ViewForeignInvitations,
    WriteUserStaffNote,
    EditUserStaffNotes,
    ViewForeignUserStaffNotes,
    ViewRegistrationIps,
    ManageAdventCalendars,
}

/// User information displayed as a list, that a user can hide with their paranoia settings.
/// Each variant matches the [`DisplayableUserStats`] variant counting the same information:
/// a list is hidden as soon as its count is hidden.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "displayable_user_lists_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum HideableUserList {
    Torrents,
    Snatched,
    ForumPosts,
    TitleGroupComments,
    RequestComments,
}

impl HideableUserList {
    pub fn matching_stat(self) -> DisplayableUserStats {
        match self {
            HideableUserList::Torrents => DisplayableUserStats::Torrents,
            HideableUserList::Snatched => DisplayableUserStats::Snatched,
            HideableUserList::ForumPosts => DisplayableUserStats::ForumPosts,
            HideableUserList::TitleGroupComments => DisplayableUserStats::TitleGroupComments,
            HideableUserList::RequestComments => DisplayableUserStats::RequestComments,
        }
    }
}

/// Implemented by every model carrying the paranoia settings of a user, so that the rule
/// deciding whether a list is hidden is written a single time.
pub trait ParanoiaHiddenInformation {
    fn paranoia_hidden_stats(&self) -> &[DisplayableUserStats];
    fn paranoia_hidden_lists(&self) -> &[HideableUserList];

    /// A list is hidden either explicitly, or because the count it belongs to is hidden.
    fn is_list_hidden(&self, list: HideableUserList) -> bool {
        self.paranoia_hidden_lists().contains(&list)
            || self.paranoia_hidden_stats().contains(&list.matching_stat())
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Register {
    pub username: String,
    pub password: String,
    pub password_verify: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Login {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,
    pub exp: i64,
    pub iat: i64,
    /// an access token only authenticates requests, a refresh token only issues new tokens
    pub typ: TokenType,
    /// only set on refresh tokens, identifies them so that each one is redeemed once
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct RefreshToken {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedUser {
    pub avatar: Option<String>,
    pub email: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserChangedPassword {
    pub current_password: String,
    pub new_password: String,
    pub new_password_verify: String,
}

/// A password reset link generated by a staff member for another user. The token it contains
/// allows its bearer to set a new password without being authenticated.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GeneratedPasswordResetToken {
    pub reset_url: String,
    #[schema(value_type = String, format = DateTime)]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResetPassword {
    pub token: String,
    pub new_password: String,
    pub new_password_verify: String,
}

/// A user's profile as seen by another user. Every statistic that the user can hide with their
/// paranoia settings is optional, and is set to `None` when the requesting user is not allowed
/// to see it.
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PublicUser {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub created_at: Option<DateTime<Utc>>,
    pub description: String,
    pub uploaded: Option<i64>,
    pub real_uploaded: Option<i64>,
    pub downloaded: Option<i64>,
    pub real_downloaded: Option<i64>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_seen: Option<DateTime<Utc>>,
    pub class_name: String,
    pub class_locked: bool,
    pub title_groups: Option<i32>,
    pub edition_groups: Option<i32>,
    pub torrents: Option<i32>,
    pub forum_posts: Option<i32>,
    pub forum_threads: Option<i32>,
    pub title_group_comments: Option<i32>,
    pub request_comments: Option<i32>,
    pub artist_comments: Option<i64>,
    pub seeding: Option<i32>,
    pub leeching: Option<i32>,
    pub snatched: Option<i32>,
    pub seeding_size: Option<i64>,
    pub requests_filled: Option<i64>,
    pub collages_started: Option<i64>,
    pub requests_voted: Option<i64>,
    pub average_seeding_time: Option<i64>,
    pub invited: Option<i64>,
    pub invitations: Option<i16>,
    pub bonus_points: Option<i64>,
    pub banned: bool,
    pub warned: bool,
    pub custom_title: Option<String>,
    pub paranoia_hidden_stats: Vec<DisplayableUserStats>,
    pub paranoia_hidden_lists: Vec<HideableUserList>,
}

impl PublicUser {
    /// Removes every statistic the user chose to hide with their paranoia settings.
    /// `ratio` has no dedicated column: it is computed from `uploaded` and `downloaded`,
    /// so hiding it also requires hiding those two.
    pub fn hide_paranoia_hidden_stats(&mut self) {
        // the hidden statistics are moved out of the profile while it is edited, and put back
        // afterwards, so that they do not have to be cloned
        let hidden_stats = std::mem::take(&mut self.paranoia_hidden_stats);
        for stat in &hidden_stats {
            match stat {
                DisplayableUserStats::Uploaded => self.uploaded = None,
                DisplayableUserStats::RealUploaded => self.real_uploaded = None,
                DisplayableUserStats::Downloaded => self.downloaded = None,
                DisplayableUserStats::RealDownloaded => self.real_downloaded = None,
                DisplayableUserStats::TitleGroups => self.title_groups = None,
                DisplayableUserStats::EditionGroups => self.edition_groups = None,
                DisplayableUserStats::Torrents => self.torrents = None,
                DisplayableUserStats::ForumPosts => self.forum_posts = None,
                DisplayableUserStats::ForumThreads => self.forum_threads = None,
                DisplayableUserStats::TitleGroupComments => self.title_group_comments = None,
                DisplayableUserStats::RequestComments => self.request_comments = None,
                DisplayableUserStats::ArtistComments => self.artist_comments = None,
                DisplayableUserStats::Seeding => self.seeding = None,
                DisplayableUserStats::Leeching => self.leeching = None,
                DisplayableUserStats::Snatched => self.snatched = None,
                DisplayableUserStats::SeedingSize => self.seeding_size = None,
                DisplayableUserStats::RequestsFilled => self.requests_filled = None,
                DisplayableUserStats::CollagesStarted => self.collages_started = None,
                DisplayableUserStats::RequestsVoted => self.requests_voted = None,
                DisplayableUserStats::AverageSeedingTime => self.average_seeding_time = None,
                DisplayableUserStats::Invited => self.invited = None,
                DisplayableUserStats::Invitations => self.invitations = None,
                DisplayableUserStats::BonusPoints => self.bonus_points = None,
                DisplayableUserStats::JoinedAt => self.created_at = None,
                DisplayableUserStats::LastSeen => self.last_seen = None,
                DisplayableUserStats::Ratio => {
                    self.uploaded = None;
                    self.downloaded = None;
                }
                // those statistics are not part of a public profile
                DisplayableUserStats::FreeleechTokens
                | DisplayableUserStats::CurrentStreak
                | DisplayableUserStats::HighestStreak => {}
            }
        }
        self.paranoia_hidden_stats = hidden_stats;
    }
}

impl ParanoiaHiddenInformation for PublicUser {
    fn paranoia_hidden_stats(&self) -> &[DisplayableUserStats] {
        &self.paranoia_hidden_stats
    }

    fn paranoia_hidden_lists(&self) -> &[HideableUserList] {
        &self.paranoia_hidden_lists
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserParanoiaSettings {
    pub paranoia_hidden_stats: Vec<DisplayableUserStats>,
    pub paranoia_hidden_lists: Vec<HideableUserList>,
}

impl ParanoiaHiddenInformation for UserParanoiaSettings {
    fn paranoia_hidden_stats(&self) -> &[DisplayableUserStats] {
        &self.paranoia_hidden_stats
    }

    fn paranoia_hidden_lists(&self) -> &[HideableUserList] {
        &self.paranoia_hidden_lists
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema, Decode)]
pub struct UserLite {
    pub id: i32,
    pub username: String,
    pub warned: bool,
    pub banned: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Decode)]
pub struct UserLiteAvatar {
    pub id: i32,
    pub username: String,
    pub class_name: String,
    pub banned: bool,
    pub avatar: Option<String>,
    pub warned: bool,
    pub custom_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Profile {
    pub user: User,
    pub torrent_clients: Vec<TorrentClient>,
    pub user_warnings: Vec<UserWarning>,
    pub last_five_uploaded_torrents: Vec<TitleGroupHierarchyLite>,
    pub last_five_snatched_torrents: Vec<TitleGroupHierarchyLite>,
    pub earned_badges: Vec<UserEarnedBadgeWithDetails>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicProfile {
    pub user: PublicUser,
    pub last_five_uploaded_torrents: Vec<TitleGroupHierarchyLite>,
    pub last_five_snatched_torrents: Vec<TitleGroupHierarchyLite>,
    pub torrent_clients: Vec<TorrentClient>,
    pub earned_badges: Vec<UserEarnedBadgeWithDetails>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow)]
pub struct UserWarning {
    pub id: i64,
    pub user_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: String,
    pub created_by_id: i32,
    pub ban: bool, // wether or not this warning bans the user
    #[schema(value_type = Option<String>, format = DateTime)]
    pub removed_at: Option<DateTime<Utc>>,
    pub removed_by_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedUserWarning {
    pub user_id: i32,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: String,
    pub ban: bool,
}

/// The groups of endpoints an API key can be allowed to reach.
#[derive(
    Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema, PartialEq, Eq, strum::AsRefStr,
)]
#[sqlx(type_name = "api_key_scope_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum APIKeyScope {
    User,
    Torrents,
    Requests,
    Forum,
    Wiki,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct APIKey {
    pub id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_used_at: Option<DateTime<Utc>>,
    pub name: String,
    pub last_four: String,
    pub scopes: Vec<APIKeyScope>,
}

/// An API key right after its creation, the only time its value is known to the user.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatedAPIKey {
    pub api_key: APIKey,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserCreatedAPIKey {
    pub name: String,
    pub scopes: Vec<APIKeyScope>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserMinimal {
    pub id: i32,
    pub passkey: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserSettings {
    pub css_sheet_name: String,
    pub irc_site_embed_enabled: bool,
    pub show_upload_method: bool,
    pub paranoia_hidden_stats: Vec<DisplayableUserStats>,
    pub paranoia_hidden_lists: Vec<HideableUserList>,
}

/// The settings of a user, with the additional information that is only read, never written
/// with the settings.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserSettingsResponse {
    #[serde(flatten)]
    pub settings: UserSettings,
    /// Amount of torrents the user uploaded anonymously. The anonymity of the uploaded torrents
    /// is changed with the dedicated endpoint.
    pub anonymous_uploaded_torrents: i64,
    /// Amount of torrents the user uploaded without being anonymous.
    pub non_anonymous_uploaded_torrents: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUploadedTorrentsAnonymity {
    /// `true` marks every torrent uploaded by the user as anonymous, `false` marks them all as
    /// not anonymous.
    pub anonymous: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserClass {
    pub name: String,
    pub new_permissions: Vec<UserPermission>,
    pub max_snatches_per_day: Option<i32>,
    pub automatic_promotion: bool,
    pub automatic_demotion: bool,
    pub promotion_allowed_while_warned: bool,
    pub previous_user_class: Option<String>,
    pub required_account_age_in_days: i32,
    pub required_ratio: f64,
    pub required_torrent_uploads: i32,
    pub required_torrent_uploads_in_unique_title_groups: i32,
    pub required_uploaded: i64,
    pub required_torrent_snatched: i32,
    pub required_downloaded: i64,
    pub required_forum_posts: i32,
    pub required_forum_posts_in_unique_threads: i32,
    pub required_title_group_comments: i32,
    pub required_seeding_size: i64,
    pub promotion_cost_bonus_points: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedUserClass {
    pub name: String,
    pub new_permissions: Vec<UserPermission>,
    pub max_snatches_per_day: Option<i32>,
    pub automatic_promotion: bool,
    pub automatic_demotion: bool,
    pub promotion_allowed_while_warned: bool,
    pub previous_user_class: Option<String>,
    pub required_account_age_in_days: i32,
    pub required_ratio: f64,
    pub required_torrent_uploads: i32,
    pub required_torrent_uploads_in_unique_title_groups: i32,
    pub required_uploaded: i64,
    pub required_torrent_snatched: i32,
    pub required_downloaded: i64,
    pub required_forum_posts: i32,
    pub required_forum_posts_in_unique_threads: i32,
    pub required_title_group_comments: i32,
    pub required_seeding_size: i64,
    pub promotion_cost_bonus_points: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedUserClass {
    pub name: String,
    pub new_permissions: Vec<UserPermission>,
    pub max_snatches_per_day: Option<i32>,
    pub automatic_promotion: bool,
    pub automatic_demotion: bool,
    pub promotion_allowed_while_warned: bool,
    pub previous_user_class: Option<String>,
    pub required_account_age_in_days: i32,
    pub required_ratio: f64,
    pub required_torrent_uploads: i32,
    pub required_torrent_uploads_in_unique_title_groups: i32,
    pub required_uploaded: i64,
    pub required_torrent_snatched: i32,
    pub required_downloaded: i64,
    pub required_forum_posts: i32,
    pub required_forum_posts_in_unique_threads: i32,
    pub required_title_group_comments: i32,
    pub required_seeding_size: i64,
    pub promotion_cost_bonus_points: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdatedUserPermissions {
    pub permissions: Vec<UserPermission>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserClassLockStatus {
    pub class_locked: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserClassChange {
    pub class_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteUserClass {
    pub target_class_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserCustomTitle {
    pub custom_title: Option<String>,
}

/// A user as seen in the user search. Like in [`PublicUser`], every statistic that the user can
/// hide with their paranoia settings is set to `None` when the requesting user may not see it.
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserSearchResult {
    pub id: i32,
    pub username: String,
    pub avatar: Option<String>,
    pub class_name: String,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub created_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_seen: Option<DateTime<Utc>>,
    pub uploaded: Option<i64>,
    pub downloaded: Option<i64>,
    pub torrents: Option<i32>,
    pub title_groups: Option<i32>,
    pub title_group_comments: Option<i32>,
    pub forum_posts: Option<i32>,
    pub forum_threads: Option<i32>,
    pub bonus_points: Option<i64>,
    pub seeding: Option<i32>,
    pub title_group_tags_applied: i64,
    pub warned: bool,
    pub banned: bool,
    pub paranoia_hidden_stats: Vec<DisplayableUserStats>,
}

impl UserSearchResult {
    /// Removes every statistic the user chose to hide with their paranoia settings. Only the
    /// statistics displayed in the user search are concerned, the other ones are ignored.
    pub fn hide_paranoia_hidden_stats(&mut self) {
        // the hidden statistics are moved out of the result while it is edited, and put back
        // afterwards, so that they do not have to be cloned
        let hidden_stats = std::mem::take(&mut self.paranoia_hidden_stats);
        for stat in &hidden_stats {
            match stat {
                DisplayableUserStats::JoinedAt => self.created_at = None,
                DisplayableUserStats::LastSeen => self.last_seen = None,
                DisplayableUserStats::Uploaded => self.uploaded = None,
                DisplayableUserStats::Downloaded => self.downloaded = None,
                DisplayableUserStats::Torrents => self.torrents = None,
                DisplayableUserStats::TitleGroups => self.title_groups = None,
                DisplayableUserStats::TitleGroupComments => self.title_group_comments = None,
                DisplayableUserStats::ForumPosts => self.forum_posts = None,
                DisplayableUserStats::ForumThreads => self.forum_threads = None,
                DisplayableUserStats::BonusPoints => self.bonus_points = None,
                DisplayableUserStats::Seeding => self.seeding = None,
                // `ratio` is computed from `uploaded` and `downloaded`, hiding it hides those two
                DisplayableUserStats::Ratio => {
                    self.uploaded = None;
                    self.downloaded = None;
                }
                _ => {}
            }
        }
        self.paranoia_hidden_stats = hidden_stats;
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SearchUsersQuery {
    pub username: Option<String>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub registered_after: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub registered_before: Option<DateTime<Utc>>,
    pub permissions: Option<Vec<UserPermission>>,
    pub permissions_match: Option<UserPermissionMatchMode>,
    pub order_by: UserSearchOrderBy,
    pub order_by_direction: OrderByDirection,
    pub page: u32,
    pub page_size: u32,
}

/// How the users of a search are matched against the searched permissions.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum UserPermissionMatchMode {
    /// The user has at least one of the searched permissions.
    #[default]
    Any,
    /// The user has every one of the searched permissions.
    All,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, strum::Display)]
pub enum UserSearchOrderBy {
    #[serde(rename = "username")]
    #[strum(serialize = "username")]
    Username,
    #[serde(rename = "created_at")]
    #[strum(serialize = "created_at")]
    CreatedAt,
    #[serde(rename = "uploaded")]
    #[strum(serialize = "uploaded")]
    Uploaded,
    #[serde(rename = "downloaded")]
    #[strum(serialize = "downloaded")]
    Downloaded,
    #[serde(rename = "torrents")]
    #[strum(serialize = "torrents")]
    Torrents,
    #[serde(rename = "title_groups")]
    #[strum(serialize = "title_groups")]
    TitleGroups,
    #[serde(rename = "title_group_comments")]
    #[strum(serialize = "title_group_comments")]
    TitleGroupComments,
    #[serde(rename = "forum_posts")]
    #[strum(serialize = "forum_posts")]
    ForumPosts,
    #[serde(rename = "forum_threads")]
    #[strum(serialize = "forum_threads")]
    ForumThreads,
    #[serde(rename = "bonus_points")]
    #[strum(serialize = "bonus_points")]
    BonusPoints,
    #[serde(rename = "seeding")]
    #[strum(serialize = "seeding")]
    Seeding,
    #[serde(rename = "last_seen")]
    #[strum(serialize = "last_seen")]
    LastSeen,
    #[serde(rename = "title_group_tags_applied")]
    #[strum(serialize = "title_group_tags_applied")]
    TitleGroupTagsApplied,
}

/// User stats used for promotion/demotion checks
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserWithStats {
    pub id: i32,
    pub class_name: String,
    pub class_locked: bool,
    pub warned: bool,
    pub bonus_points: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub uploaded: i64,
    pub downloaded: i64,
    pub snatched: i32,
    pub forum_posts: i32,
    pub seeding_size: i64,
    pub torrent_uploads: i32,
    pub torrent_uploads_in_unique_title_groups: i32,
    pub title_group_comments: i32,
    pub forum_posts_in_unique_threads: i32,
}
