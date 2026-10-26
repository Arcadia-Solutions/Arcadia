use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::Display;
use utoipa::{IntoParams, ToSchema};

use super::{common::OrderByDirection, user::UserLiteAvatar};

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Donation {
    pub id: i64,
    pub donated_by_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub donated_at: DateTime<Utc>,
    pub created_by_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub amount: f64,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DonationSearchResult {
    pub id: i64,
    pub donated_by_id: i32,
    pub donated_by: UserLiteAvatar,
    #[schema(value_type = String, format = DateTime)]
    pub donated_at: DateTime<Utc>,
    pub created_by_id: i32,
    pub created_by: UserLiteAvatar,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub amount: f64,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SearchDonationsQuery {
    pub donated_by_id: Option<i32>,
    pub created_by_id: Option<i32>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    #[serde(default)]
    #[schema(value_type = Option<String>, format = DateTime)]
    #[param(value_type = Option<String>)]
    pub donated_at_start: Option<DateTime<Utc>>,
    #[serde(default)]
    #[schema(value_type = Option<String>, format = DateTime)]
    #[param(value_type = Option<String>)]
    pub donated_at_end: Option<DateTime<Utc>>,
    #[serde(default = "default_order_by")]
    pub order_by_column: DonationOrderBy,
    #[serde(default = "default_order_direction")]
    pub order_by_direction: OrderByDirection,
    pub page: u32,
    pub page_size: u32,
}

fn default_order_by() -> DonationOrderBy {
    DonationOrderBy::DonatedAt
}

fn default_order_direction() -> OrderByDirection {
    OrderByDirection::Desc
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DonationOrderBy {
    DonatedAt,
    CreatedAt,
    Amount,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchDonationsResponse {
    pub results: Vec<DonationSearchResult>,
    pub page: u32,
    pub page_size: u32,
    pub total_items: i64,
    pub total_amount: f64,
    pub unique_donors_count: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UserCreatedDonation {
    pub donated_by_id: i32,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub donated_at: Option<DateTime<Utc>>,
    pub amount: f64,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditedDonation {
    pub id: i64,
    pub donated_by_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub donated_at: DateTime<Utc>,
    pub amount: f64,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeletedDonation {
    pub id: i64,
}
