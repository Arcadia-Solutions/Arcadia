use crate::{
    connection_pool::ConnectionPool,
    models::{
        bonus_points_log::{
            BonusPointsLog, BonusPointsLogAction, BonusPointsLogOrderByColumn,
            SearchBonusPointsLogsQuery,
        },
        common::{OrderByDirection, PaginatedResults},
    },
};
use arcadia_common::error::Result;
use sqlx::{Postgres, Transaction};

impl ConnectionPool {
    pub async fn log_bonus_points_change_tx(
        tx: &mut Transaction<'_, Postgres>,
        user_id: i32,
        action: BonusPointsLogAction,
        amount: i64,
        details: Option<&str>,
        item_id: Option<i64>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO bonus_points_logs (user_id, action, amount, details, item_id)
                VALUES ($1, $2, $3, $4, $5)
            "#,
            user_id,
            action as BonusPointsLogAction,
            amount,
            details,
            item_id
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn award_bonus_points(
        &self,
        user_id: i32,
        action: BonusPointsLogAction,
        amount: i64,
        details: Option<&str>,
        item_id: Option<i64>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
                UPDATE users
                SET bonus_points = bonus_points + $2
                WHERE id = $1
            "#,
            user_id,
            amount
        )
        .execute(&mut *tx)
        .await?;

        Self::log_bonus_points_change_tx(&mut tx, user_id, action, amount, details, item_id)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn search_bonus_points_logs(
        &self,
        query: &SearchBonusPointsLogsQuery,
    ) -> Result<PaginatedResults<BonusPointsLog>> {
        let limit = query.page_size as i64;
        let offset = ((query.page - 1) * query.page_size) as i64;

        let order_by_column = match query.order_by_column {
            BonusPointsLogOrderByColumn::CreatedAt => "created_at",
            BonusPointsLogOrderByColumn::Amount => "amount",
            BonusPointsLogOrderByColumn::Action => "action",
        };
        let direction = match query.order_by_direction {
            OrderByDirection::Asc => "ASC",
            OrderByDirection::Desc => "DESC",
        };

        let actions: Vec<BonusPointsLogAction> = query
            .actions
            .split(',')
            .filter(|part| !part.is_empty())
            .filter_map(|part| {
                serde_json::from_value(serde_json::Value::String(part.to_owned())).ok()
            })
            .collect();

        let list_query = format!(
            r#"
                SELECT created_at, user_id, action, amount, details, item_id
                FROM bonus_points_logs
                WHERE user_id = $1
                  AND created_at >= $2
                  AND created_at <= $3
                  AND (cardinality($4::bonus_points_log_action_enum[]) = 0 OR action = ANY($4))
                ORDER BY {order_by_column} {direction}, created_at DESC
                LIMIT $5 OFFSET $6
            "#
        );

        let results: Vec<BonusPointsLog> = sqlx::query_as(sqlx::AssertSqlSafe(list_query))
            .bind(query.user_id)
            .bind(query.from_date)
            .bind(query.to_date)
            .bind(&actions)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let total_items = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) AS "count!"
                FROM bonus_points_logs
                WHERE user_id = $1
                  AND created_at >= $2
                  AND created_at <= $3
                  AND (cardinality($4::bonus_points_log_action_enum[]) = 0 OR action = ANY($4))
            "#,
            query.user_id,
            query.from_date,
            query.to_date,
            actions.as_slice() as &[BonusPointsLogAction]
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }
}
