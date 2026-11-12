use arcadia_common::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use super::{common::OrderByDirection, title_group::TitleGroupHierarchyLite, user::UserLiteAvatar};

/// A user's rating (percentage from 0 to 100) of a title group, optionally explained by a
/// BBCode comment. A user has at most one per title group.
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TitleGroupRating {
    pub id: i64,
    pub title_group_id: i32,
    pub user_id: i32,
    pub rating: i16,
    pub comment: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedTitleGroupRating {
    pub title_group_id: i32,
    pub rating: i16,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedTitleGroupRating {
    pub rating: i16,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TitleGroupRatingHierarchy {
    pub id: i64,
    pub rating: i16,
    pub comment: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    pub created_by: UserLiteAvatar,
}

/// A rating displayed outside of its title group, on the profile of its author.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TitleGroupRatingWithTitleGroup {
    pub id: i64,
    pub title_group: TitleGroupHierarchyLite,
    pub user_id: i32,
    pub rating: i16,
    pub comment: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

/// Query of the paginated ratings written by a user, filtered like [`TitleGroupRatingSearchQuery`].
#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct UserTitleGroupRatingSearchQuery {
    pub user_id: i32,
    pub minimum_rating: i16,
    pub maximum_rating: i16,
    pub content: String,
    pub order_by_column: TitleGroupRatingSearchOrderByColumn,
    pub order_by_direction: OrderByDirection,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, strum::Display)]
pub enum TitleGroupRatingSearchOrderByColumn {
    #[serde(rename = "created_at")]
    #[strum(serialize = "created_at")]
    CreatedAt,
    #[serde(rename = "rating")]
    #[strum(serialize = "rating")]
    Rating,
}

/// Query of the paginated ratings of a title group whose rating is within
/// `minimum_rating..=maximum_rating` and whose comment contains `content` (case insensitive, empty
/// matches every rating).
#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TitleGroupRatingSearchQuery {
    pub title_group_id: i32,
    pub minimum_rating: i16,
    pub maximum_rating: i16,
    pub content: String,
    pub order_by_column: TitleGroupRatingSearchOrderByColumn,
    pub order_by_direction: OrderByDirection,
    pub page: u32,
    pub page_size: u32,
}

pub fn validate_rating(rating: i16) -> Result<()> {
    if (0..=100).contains(&rating) {
        Ok(())
    } else {
        Err(Error::TitleGroupRatingOutOfRange)
    }
}
