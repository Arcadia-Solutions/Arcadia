use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::ipnetwork::IpNetwork};
use strum::Display;
use utoipa::ToSchema;

use crate::models::user::UserLite;

#[derive(Debug, Serialize, Deserialize, Clone, Display, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_application_status_enum", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserApplicationStatus {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserApplication {
    pub id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub body: String,
    pub email: String,
    pub referral: String,
    #[schema(value_type = String)]
    pub applied_from_ip: IpNetwork,
    pub staff_note: String,
    pub status: UserApplicationStatus,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserCreatedUserApplication {
    pub body: String,
    pub email: String,
    pub referral: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserApplicationHierarchy {
    pub id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub body: String,
    pub email: String,
    pub referral: String,
    #[schema(value_type = String)]
    pub applied_from_ip: IpNetwork,
    pub staff_note: String,
    pub status: UserApplicationStatus,
    pub user: Option<UserLite>,
}
