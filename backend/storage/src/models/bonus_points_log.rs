use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::Display;
use utoipa::{IntoParams, ToSchema};

use super::common::OrderByDirection;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "bonus_points_log_action_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BonusPointsLogAction {
    SnatchCostDeduction,
    SnatchCostReceivedAsUploader,
    SnatchCostReceivedAsSeeder,
    TorrentUploadReward,
    TorrentRequestVoteSpent,
    TorrentRequestFillReward,
    TorrentRequestVoteRefund,
    GiftSent,
    GiftReceived,
    SeedtimeReward,
    SideEffectReward,
    ShopPurchaseUpload,
    ShopPurchaseFreeleechTokens,
    ShopPurchasePromotion,
    AdventCalendarDoorReward,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BonusPointsLog {
    #[schema(value_type = String, format = DateTime)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub user_id: i32,
    pub action: BonusPointsLogAction,
    pub amount: i64,
    pub details: Option<String>,
    pub item_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, Display)]
pub enum BonusPointsLogOrderByColumn {
    #[serde(rename = "created_at")]
    #[strum(serialize = "created_at")]
    CreatedAt,
    #[serde(rename = "amount")]
    #[strum(serialize = "amount")]
    Amount,
    #[serde(rename = "action")]
    #[strum(serialize = "action")]
    Action,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SearchBonusPointsLogsQuery {
    pub page: u32,
    pub page_size: u32,
    pub order_by_column: BonusPointsLogOrderByColumn,
    pub order_by_direction: OrderByDirection,
    #[schema(value_type = String, format = DateTime)]
    #[param(value_type = String)]
    pub from_date: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = DateTime)]
    #[param(value_type = String)]
    pub to_date: chrono::DateTime<chrono::Utc>,
    pub actions: String,
    /// user whose logs are searched.
    /// searching the logs of another user requires the see_foreign_bonus_points_logs permission
    pub user_id: i32,
}
