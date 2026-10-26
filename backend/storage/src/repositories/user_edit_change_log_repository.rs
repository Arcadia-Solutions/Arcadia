use crate::{
    connection_pool::ConnectionPool,
    models::{
        common::PaginatedResults,
        user::UserLiteAvatar,
        user_edit_change_log::{
            NewUserEditChangeLog, SearchUserEditChangeLogsQuery, UserEditChangeLogResult,
        },
    },
};
use arcadia_common::error::{Error, Result};

impl ConnectionPool {
    pub async fn create_user_edit_change_log(&self, log: &NewUserEditChangeLog) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_edit_change_logs (item_type, item_id, edited_by_id, edits)
            VALUES ($1, $2, $3, $4)
            "#,
            log.item_type,
            log.item_id,
            log.edited_by_id,
            log.edits
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotCreateUserEditChangeLog)?;

        Ok(())
    }

    pub async fn delete_user_edit_change_log(&self, id: i64) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM user_edit_change_logs WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteUserEditChangeLog)?;

        Ok(())
    }

    pub async fn delete_all_user_edit_change_logs(&self) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM user_edit_change_logs
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteAllUserEditChangeLogs)?;

        Ok(())
    }

    pub async fn search_user_edit_change_logs(
        &self,
        query: SearchUserEditChangeLogsQuery,
    ) -> Result<PaginatedResults<UserEditChangeLogResult>> {
        let total_items: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM user_edit_change_logs
            WHERE ($1::INT IS NULL OR edited_by_id = $1)
              AND ($2::TEXT IS NULL OR item_type = $2)
            "#,
            query.user_id,
            query.item_type
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT
                l.id,
                l.item_type,
                l.item_id,
                l.edited_at,
                l.edits,
                u.id as user_id,
                u.username,
                u.class_name,
                u.banned,
                u.avatar,
                u.warned,
                u.custom_title
            FROM user_edit_change_logs l
            JOIN users u ON l.edited_by_id = u.id
            WHERE ($1::INT IS NULL OR l.edited_by_id = $1)
              AND ($2::TEXT IS NULL OR l.item_type = $2)
            ORDER BY
              CASE WHEN $3 = 'edited_at' AND $4 = 'asc' THEN l.edited_at END ASC,
              CASE WHEN $3 = 'edited_at' AND $4 = 'desc' THEN l.edited_at END DESC
            OFFSET ($5 - 1) * LEAST($6, 100)
            LIMIT LEAST($6, 100)
            "#,
            query.user_id,
            query.item_type,
            query.sort_by_column.to_string(),
            query.sort_by_direction.to_string(),
            query.page as i32,
            query.page_size as i32
        )
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| UserEditChangeLogResult {
                id: row.id,
                item_type: row.item_type,
                item_id: row.item_id,
                edited_at: row.edited_at,
                edits: row.edits,
                edited_by: UserLiteAvatar {
                    id: row.user_id,
                    username: row.username,
                    class_name: row.class_name,
                    banned: row.banned,
                    avatar: row.avatar,
                    warned: row.warned,
                    custom_title: row.custom_title,
                },
            })
            .collect();

        Ok(PaginatedResults {
            results,
            total_items,
            page: query.page as u32,
            page_size: query.page_size as u32,
        })
    }
}
