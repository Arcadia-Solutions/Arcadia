use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

use super::torrent::TorrentSearch;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "user_badge_type_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserBadgeType {
    Manual,
    TorrentsUploaded,
    ForumPosts,
    ForumThreads,
    AdventCalendarDoors,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserBadgeCategory {
    pub id: i32,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedUserBadgeCategory {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EditedUserBadgeCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserBadge {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub category_id: i32,
    pub badge_type: UserBadgeType,
    pub is_secret: bool,
    pub revoke_when_criteria_unmet: bool,
    pub criteria: Option<serde_json::Value>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedUserBadge {
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub category_id: i32,
    pub badge_type: UserBadgeType,
    pub is_secret: bool,
    pub revoke_when_criteria_unmet: bool,
    pub criteria: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EditedUserBadge {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub category_id: i32,
    pub badge_type: UserBadgeType,
    pub is_secret: bool,
    pub revoke_when_criteria_unmet: bool,
    pub criteria: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum UserBadgeListItem {
    Visible(UserBadge),
    Hidden { id: i32, is_secret: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserEarnedBadge {
    pub id: i32,
    pub user_id: i32,
    pub badge_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub awarded_at: DateTime<Local>,
    pub awarded_by_id: Option<i32>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserEarnedBadgeWithDetails {
    pub id: i32,
    pub user_id: i32,
    pub badge_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub awarded_at: DateTime<Local>,
    pub awarded_by_id: Option<i32>,
    pub note: Option<String>,
    pub badge_name: String,
    pub badge_description: String,
    pub badge_image_url: String,
    pub badge_category_id: i32,
    pub badge_type: UserBadgeType,
    pub badge_is_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserBadgeManualAward {
    pub user_id: i32,
    pub badge_id: i32,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserBadgeCriteria {
    TorrentsUploaded {
        search: Box<TorrentSearch>,
        minimum_title_group_amount: i64,
    },
    ForumPosts {
        minimum_post_character_count: i32,
        required_substring: Option<String>,
        minimum_post_amount: i32,
    },
    ForumThreads {
        minimum_thread_name_character_count: i32,
        required_substring: Option<String>,
        minimum_thread_amount: i32,
    },
    AdventCalendarDoors {
        calendar_id: i32,
        minimum_opened_door_amount: i32,
        /// only the doors with these names are counted, every door is counted when empty
        #[serde(default)]
        door_names: Vec<String>,
    },
}
