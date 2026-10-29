use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::Display;
use utoipa::{IntoParams, ToSchema};

use super::site_highlight::SiteHighlightItemType;
use super::user::{UserLite, UserLiteAvatar};
use crate::models::reaction::ContentReaction;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct RelatedForumThread {
    pub forum_thread_id: i64,
    pub thread_name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateRelatedForumThread {
    pub item_type: SiteHighlightItemType,
    pub item_id: i64,
    pub forum_thread_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct DeleteRelatedForumThreadQuery {
    pub item_type: SiteHighlightItemType,
    pub item_id: i64,
    pub forum_thread_id: i64,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumCategory {
    pub id: i32,
    pub name: String,
    pub sort_order: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserCreatedForumCategory {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct EditedForumCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumSubCategory {
    pub id: i32,
    pub forum_category_id: i32,
    pub name: String,
    pub sort_order: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
    pub threads_amount: i64,
    pub posts_amount: i64,
    pub forbidden_classes: Vec<String>,
    pub new_threads_restricted: bool,
    pub thread_sort_by: ForumThreadSortBy,
    pub thread_sort_direction: ForumThreadSortDirection,
}

/// The value threads within a forum sub-category are sorted by. Defaults to `LatestPost`
/// and is changeable by users with the edit forum sub-category permission.
#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, sqlx::Type, ToSchema, Display, PartialEq, Eq,
)]
#[sqlx(type_name = "forum_thread_sort_by_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ForumThreadSortBy {
    LatestPost,
    CreatedAt,
    Name,
    PostsAmount,
    ViewsCount,
}

/// The direction `thread_sort_by` is applied in. Defaults to `Descending` and is changeable
/// by users with the edit forum sub-category permission.
#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, sqlx::Type, ToSchema, Display, PartialEq, Eq,
)]
#[sqlx(
    type_name = "forum_thread_sort_direction_enum",
    rename_all = "snake_case"
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ForumThreadSortDirection {
    Ascending,
    Descending,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserCreatedForumSubCategory {
    pub forum_category_id: i32,
    pub name: String,
    pub new_threads_restricted: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct EditedForumSubCategory {
    pub id: i32,
    pub name: String,
    pub new_threads_restricted: bool,
    pub thread_sort_by: ForumThreadSortBy,
    pub thread_sort_direction: ForumThreadSortDirection,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumThread {
    pub id: i64,
    pub forum_sub_category_id: i32,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
    pub posts_amount: i64,
    pub pinned: bool,
    /// only stops new replies in the thread, its author can still edit it
    pub locked: bool,
    pub views_count: i64,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct UserCreatedForumThread {
    pub forum_sub_category_id: i32,
    pub name: String,
    pub first_post: UserCreatedForumPost,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct EditedForumThread {
    pub id: i64,
    pub forum_sub_category_id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumThreadLite {
    pub id: i64,
    pub forum_sub_category_id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumPost {
    pub id: i64,
    pub forum_thread_id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Local>,
    pub created_by_id: i32,
    pub content: String,
    pub sticky: bool,
    pub locked: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct EditedForumPost {
    pub id: i64,
    pub content: String,
    pub sticky: bool,
    pub locked: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct UserCreatedForumPost {
    pub content: String,
    pub forum_thread_id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteForumCategoryQuery {
    pub id: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteForumSubCategoryQuery {
    pub id: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteForumThreadQuery {
    pub id: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeleteForumPostQuery {
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumOverview {
    forum_categories: Vec<ForumCategoryHierarchy>,
    latest_posts_in_threads: Vec<ForumSearchResult>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumCategoryHierarchy {
    pub id: i32,
    pub name: String,
    pub sub_categories: Vec<ForumSubCategoryHierarchy>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumCategoryLite {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumSubCategoryLite {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumSubCategoryHierarchy {
    pub id: i32,
    pub name: String,
    pub threads_amount: i64,
    pub posts_amount: i64,
    pub forbidden_classes: Vec<String>,
    pub new_threads_restricted: bool,
    pub thread_sort_by: ForumThreadSortBy,
    pub thread_sort_direction: ForumThreadSortDirection,
    pub is_allowed_poster: bool,
    pub is_subscribed: bool,
    pub latest_post_in_thread: Option<ForumThreadPostLite>,
    pub threads: Option<Vec<ForumThreadHierarchy>>,
    pub category: ForumCategoryLite,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumThreadHierarchy {
    pub id: i64,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by: UserLite,
    pub latest_post: ForumThreadPostLite,
    pub posts_amount: i64,
    pub pinned: bool,
    pub locked: bool,
    pub views_count: i64,
    pub ever_opened: bool,
    pub has_new_posts: bool,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumThreadPostLite {
    pub id: i64,
    pub thread_id: i64,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by: UserLite,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumThreadEnriched {
    pub id: i64,
    pub name: String,
    pub is_subscribed: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
    pub posts_amount: i64,
    pub pinned: bool,
    pub locked: bool,
    pub views_count: i64,
    pub forum_sub_category_name: String,
    pub forum_sub_category_id: i32,
    pub forum_category_name: String,
    pub forum_category_id: i32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ForumThreadEnrichedHierarchy {
    #[serde(flatten)]
    pub thread: ForumThreadEnriched,
    pub poll: Option<ForumPollHierarchy>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserCreatedForumPoll {
    pub forum_thread_id: i64,
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UserCreatedForumPollVote {
    pub forum_poll_id: i64,
    // None means a blank / abstention vote
    pub forum_poll_option_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumPoll {
    pub id: i64,
    pub forum_thread_id: i64,
    pub question: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumPollOptionResult {
    pub id: i64,
    pub content: String,
    // None until the requesting user has voted on this poll
    pub votes_amount: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ForumPollHierarchy {
    pub id: i64,
    pub question: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
    pub has_voted: bool,
    pub options: Vec<ForumPollOptionResult>,
    // count of blank (no-option) votes; None until the requesting user has voted
    pub blank_votes_amount: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumPostHierarchy {
    pub id: i64,
    pub forum_thread_id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    pub created_by: UserLiteAvatar,
    pub content: String,
    pub sticky: bool,
    pub locked: bool,
    /// Only filled when reading the posts of a thread; empty everywhere else.
    pub reactions: Vec<ContentReaction>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumPostAndThreadName {
    pub id: i64,
    pub forum_thread_id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Local>,
    pub created_by_id: i32,
    pub content: String,
    pub sticky: bool,
    pub forum_thread_name: String,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GetForumThreadPostsQuery {
    pub thread_id: i64,
    pub page: Option<u32>,
    pub page_size: u32,
    pub post_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ForumSearchResult {
    pub thread_name: String,
    pub thread_id: i64,
    pub post: String,
    pub post_id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub post_created_at: DateTime<Utc>,
    pub post_created_by_id: i32,
    pub post_created_by_username: String,
    pub sub_category_name: String,
    pub sub_category_id: i32,
    pub category_name: String,
    pub category_id: i32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ForumSearchQuery {
    pub thread_name: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

/// Query of the paginated list of every forum post written by a user.
#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ForumPostSearchQuery {
    pub created_by_id: i32,
    pub page: u32,
    pub page_size: u32,
}

/// A forum post displayed outside of its thread: it carries the thread, sub category and
/// category it was written in.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ForumPostWithLocation {
    #[serde(flatten)]
    pub post: ForumPostHierarchy,
    pub thread_name: String,
    pub sub_category_id: i32,
    pub sub_category_name: String,
    pub category_id: i32,
    pub category_name: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ForumSubCategoryAllowedPoster {
    pub forum_sub_category_id: i32,
    pub user_id: i32,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GetForumSubCategoryAllowedPostersQuery {
    pub forum_sub_category_id: i32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PinForumThread {
    pub pin: bool,
    pub thread_id: i64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ReorderForumCategoryEntry {
    pub id: i32,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ReorderForumCategories {
    pub categories: Vec<ReorderForumCategoryEntry>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ReorderForumSubCategoryEntry {
    pub id: i32,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ReorderForumSubCategories {
    pub forum_category_id: i32,
    pub sub_categories: Vec<ReorderForumSubCategoryEntry>,
}
