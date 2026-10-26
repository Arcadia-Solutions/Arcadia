use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use super::common::OrderByDirection;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Series {
    pub id: i64,
    pub name: String,
    pub description: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Local>,
    pub created_by_id: i32,
    pub covers: Vec<String>,
    pub banners: Option<Vec<String>>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeriesEnriched {
    pub series: Series,
    pub related_threads: Vec<crate::models::forum::RelatedForumThread>,
    pub title_group_stats: crate::models::title_group_stats::TitleGroupStatsResponse,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedSeries {
    pub name: String,
    pub description: String,
    pub covers: Vec<String>,
    pub banners: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedSeries {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub covers: Vec<String>,
    pub banners: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, sqlx::FromRow, Clone)]
pub struct SeriesLite {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SeriesSearchResult {
    pub id: i64,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Local>,
    pub created_by_id: i32,
    pub covers: Vec<String>,
    pub banners: Option<Vec<String>>,
    pub tags: Vec<String>,
    pub title_groups_amount: i64,
    // pub newest_title_group: Option<TitleGroupInfoLite>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, strum::Display)]
pub enum SeriesSearchOrderByColumn {
    #[serde(rename = "name")]
    #[strum(serialize = "name")]
    Name,
    #[serde(rename = "title_groups_amount")]
    #[strum(serialize = "title_groups_amount")]
    TitleGroupsAmount,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SearchSeriesQuery {
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub order_by_column: SeriesSearchOrderByColumn,
    pub order_by_direction: OrderByDirection,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeriesSearchResponse {
    pub results: Vec<SeriesSearchResult>,
    pub total_items: i64,
}
