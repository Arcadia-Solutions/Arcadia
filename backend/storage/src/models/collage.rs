use crate::models::user::UserLite;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(type_name = "collage_category_enum")]
pub enum CollageCategory {
    Personal,
    #[sqlx(rename = "Staff Picks")]
    #[serde(rename = "Staff Picks")]
    StaffPicks,
    External, // replicates a collage from somewhere else (yt playlist, magazine's picks, etc.)
    Theme,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Collage {
    pub id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
    pub name: String,
    pub cover: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub category: CollageCategory,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CollageEnriched {
    pub collage: Collage,
    pub is_subscribed: bool,
    pub title_group_stats: crate::models::title_group_stats::TitleGroupStatsResponse,
}
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserCreatedCollage {
    pub name: String,
    pub cover: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub category: CollageCategory,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedCollage {
    pub id: i64,
    pub name: String,
    pub cover: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub category: CollageCategory,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct DeleteCollageQuery {
    pub collage_id: i64,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct DeleteCollageEntriesQuery {
    pub collage_id: i64,
    pub title_group_id: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CollageEntry {
    pub id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
    pub title_group_id: i32,
    pub collage_id: i64,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserCreatedCollageEntry {
    pub title_group_id: i32,
    pub collage_id: i64,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CollageSearchResult {
    pub id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
    pub created_by: UserLite,
    pub name: String,
    pub cover: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub category: CollageCategory,
    pub entries_amount: i64,
    #[schema(value_type = String, format = DateTime)]
    pub last_entry_at: Option<DateTime<Local>>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SearchCollagesQuery {
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SearchCollagesLiteQuery {
    pub name: String,
    pub results_amount: u16,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct CollageLite {
    pub id: i64,
    pub name: String,
    pub cover: Option<String>,
}
