use arcadia_common::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use super::{common::OrderByDirection, title_group::TitleGroupHierarchyLite, user::UserLiteAvatar};

/// A user's rating (percentage from 0 to 100) of a title group, optionally explained by a
/// BBCode review. A user has at most one per title group.
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TitleGroupReview {
    pub id: i64,
    pub title_group_id: i32,
    pub user_id: i32,
    pub rating: i16,
    pub review: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedTitleGroupReview {
    pub title_group_id: i32,
    pub rating: i16,
    pub review: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedTitleGroupReview {
    pub rating: i16,
    pub review: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TitleGroupReviewHierarchy {
    pub id: i64,
    pub rating: i16,
    pub review: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    pub created_by: UserLiteAvatar,
}

/// A review displayed outside of its title group, on the profile of its author.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TitleGroupReviewWithTitleGroup {
    pub id: i64,
    pub title_group: TitleGroupHierarchyLite,
    pub user_id: i32,
    pub rating: i16,
    pub review: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// Query of the paginated reviews written by a user, filtered like [`TitleGroupReviewSearchQuery`].
#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct UserTitleGroupReviewSearchQuery {
    pub user_id: i32,
    #[serde(default)]
    pub minimum_rating: i16,
    #[serde(default = "default_maximum_rating")]
    pub maximum_rating: i16,
    #[serde(default)]
    pub content: String,
    pub order_by_column: TitleGroupReviewSearchOrderByColumn,
    pub order_by_direction: OrderByDirection,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, strum::Display)]
pub enum TitleGroupReviewSearchOrderByColumn {
    #[serde(rename = "created_at")]
    #[strum(serialize = "created_at")]
    CreatedAt,
    #[serde(rename = "rating")]
    #[strum(serialize = "rating")]
    Rating,
}

/// Query of the paginated reviews of a title group whose rating is within
/// `minimum_rating..=maximum_rating` and whose text contains `content` (case insensitive, empty
/// matches every review).
#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TitleGroupReviewSearchQuery {
    pub title_group_id: i32,
    #[serde(default)]
    pub minimum_rating: i16,
    #[serde(default = "default_maximum_rating")]
    pub maximum_rating: i16,
    #[serde(default)]
    pub content: String,
    pub order_by_column: TitleGroupReviewSearchOrderByColumn,
    pub order_by_direction: OrderByDirection,
    pub page: u32,
    pub page_size: u32,
}

fn default_maximum_rating() -> i16 {
    100
}

pub fn validate_rating(rating: i16) -> Result<()> {
    if (0..=100).contains(&rating) {
        Ok(())
    } else {
        Err(Error::TitleGroupReviewRatingOutOfRange)
    }
}
