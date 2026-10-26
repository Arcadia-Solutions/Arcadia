use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::Display;
use utoipa::ToSchema;

use super::common::OrderByDirection;
use super::user::UserLiteAvatar;

pub struct NewUserEditChangeLog {
    pub item_type: String,
    pub item_id: i64,
    pub edited_by_id: i32,
    pub edits: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserEditChangeLogResult {
    pub id: i64,
    pub item_type: String,
    pub item_id: i64,
    #[schema(value_type = String, format = DateTime)]
    pub edited_at: DateTime<Utc>,
    pub edits: serde_json::Value,
    pub edited_by: UserLiteAvatar,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UserEditChangeLogSortByColumn {
    EditedAt,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchUserEditChangeLogsQuery {
    pub user_id: Option<i32>,
    pub item_type: Option<String>,
    pub sort_by_column: UserEditChangeLogSortByColumn,
    pub sort_by_direction: OrderByDirection,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DeleteUserEditChangeLogQuery {
    pub id: i64,
}
