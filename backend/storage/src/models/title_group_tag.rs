use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::Display;
use utoipa::{IntoParams, ToSchema};

use crate::models::{
    common::OrderByDirection,
    title_group::{ContentType, TitleGroupHierarchyLite},
    user::UserLite,
};

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct TitleGroupTag {
    pub id: i32,
    pub name: String,
    pub synonyms: Vec<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by_id: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UserCreatedTitleGroupTag {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedTitleGroupTag {
    pub id: i32,
    pub name: String,
    pub synonyms: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TitleGroupTagLite {
    pub name: String,
    pub synonyms: Vec<String>,
    pub id: i32,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct TitleGroupTagEnriched {
    pub id: i32,
    pub name: String,
    pub synonyms: Vec<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by: UserLite,
    pub uses: i32,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<UserLite>,
    pub deletion_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Display)]
pub enum TitleGroupTagSearchOrderByColumn {
    #[serde(rename = "created_at")]
    #[strum(serialize = "created_at")]
    CreatedAt,
    #[serde(rename = "uses")]
    #[strum(serialize = "uses")]
    Uses,
    #[serde(rename = "name")]
    #[strum(serialize = "name")]
    Name,
    #[serde(rename = "created_by")]
    #[strum(serialize = "created_by")]
    CreatedBy,
    #[serde(rename = "deleted_at")]
    #[strum(serialize = "deleted_at")]
    DeletedAt,
    #[serde(rename = "deleted_by")]
    #[strum(serialize = "deleted_by")]
    DeletedBy,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct SearchTitleGroupTagsQuery {
    pub name: String,
    /// when true, the deleted tags are returned alongside the live ones
    pub show_deleted: bool,
    // pagination and ordering
    pub page: u32,
    pub page_size: u32,
    pub order_by_column: TitleGroupTagSearchOrderByColumn,
    pub order_by_direction: OrderByDirection,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TitleGroupAppliedTag {
    pub tag_id: i32,
    pub tag_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by: UserLite,
    /// amount of tags applied by `created_by` on any title group, within the requested time range
    pub user_applied_tags_amount: i64,
    /// `tags` holds every tag applied to the title group, `edition_groups` is always empty
    pub title_group: TitleGroupHierarchyLite,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Display)]
pub enum TitleGroupAppliedTagOrderByColumn {
    #[serde(rename = "tag_name")]
    #[strum(serialize = "tag_name")]
    TagName,
    #[serde(rename = "username")]
    #[strum(serialize = "username")]
    Username,
    #[serde(rename = "title_group_name")]
    #[strum(serialize = "title_group_name")]
    TitleGroupName,
    #[serde(rename = "created_at")]
    #[strum(serialize = "created_at")]
    CreatedAt,
    #[serde(rename = "title_group_tags_amount")]
    #[strum(serialize = "title_group_tags_amount")]
    TitleGroupTagsAmount,
    #[serde(rename = "content_type")]
    #[strum(serialize = "content_type")]
    ContentType,
    #[serde(rename = "user_applied_tags_amount")]
    #[strum(serialize = "user_applied_tags_amount")]
    UserAppliedTagsAmount,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SearchTitleGroupAppliedTagsQuery {
    /// exact name of the tag
    pub tag_name: Option<String>,
    /// partial username of the user who applied the tag
    pub username: Option<String>,
    /// partial name of the title group
    pub title_group_name: Option<String>,
    pub content_type: Option<ContentType>,
    /// start of the time range used to compute `user_applied_tags_amount`, does not filter the results
    #[schema(value_type = Option<String>, format = DateTime)]
    #[param(value_type = Option<String>, format = DateTime)]
    pub user_applied_tags_after: Option<DateTime<Utc>>,
    /// end of the time range used to compute `user_applied_tags_amount`, does not filter the results
    #[schema(value_type = Option<String>, format = DateTime)]
    #[param(value_type = Option<String>, format = DateTime)]
    pub user_applied_tags_before: Option<DateTime<Utc>>,
    // pagination and ordering
    pub page: u32,
    pub page_size: u32,
    pub order_by_column: TitleGroupAppliedTagOrderByColumn,
    pub order_by_direction: OrderByDirection,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteTitleGroupTagRequest {
    pub id: i32,
    pub deletion_reason: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RestoreTitleGroupTagRequest {
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MergeTitleGroupTagsRequest {
    /// the tag that is merged away: its title groups and its names go to the
    /// target tag, and it ends up soft deleted
    pub source_tag_id: i32,
    pub target_tag_id: i32,
}
