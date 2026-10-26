use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use super::user::{UserLite, UserLiteAvatar};

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TitleGroupComment {
    pub id: i64,
    pub content: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Local>,
    pub created_by_id: i32,
    pub title_group_id: i32,
    pub locked: bool,
    pub refers_to_torrent_id: Option<i32>,
    pub answers_to_comment_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedTitleGroupComment {
    pub content: String,
    pub title_group_id: i32,
    pub refers_to_torrent_id: Option<i32>,
    pub answers_to_comment_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedTitleGroupComment {
    pub content: String,
    pub locked: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TitleGroupCommentHierarchy {
    pub id: i64,
    pub content: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Local>,
    pub created_by_id: i32,
    pub title_group_id: i32,
    pub locked: bool,
    pub refers_to_torrent_id: Option<i32>,
    pub answers_to_comment_id: Option<i64>,
    pub created_by: UserLiteAvatar,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TitleGroupCommentSearchQuery {
    pub content: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

/// Query of the paginated list of every title group comment written by a user.
#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct UserTitleGroupCommentSearchQuery {
    pub created_by_id: i32,
    pub page: u32,
    pub page_size: u32,
}

/// A title group comment displayed outside of its title group: it carries the title group it
/// was written on.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TitleGroupCommentWithLocation {
    #[serde(flatten)]
    pub comment: TitleGroupCommentHierarchy,
    pub title_group_name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TitleGroupCommentSearchResult {
    pub id: i64,
    pub content: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by: UserLite,
    pub title_group_id: i32,
    pub title_group_name: String,
}
