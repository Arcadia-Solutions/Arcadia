use crate::{
    connection_pool::ConnectionPool,
    models::{
        common::PaginatedResults,
        notification::NotificationEvent,
        title_group_comment::{
            EditedTitleGroupComment, TitleGroupComment, TitleGroupCommentHierarchy,
            TitleGroupCommentSearchQuery, TitleGroupCommentSearchResult,
            TitleGroupCommentWithLocation, UserCreatedTitleGroupComment,
            UserTitleGroupCommentSearchQuery,
        },
        user::{UserLite, UserLiteAvatar},
    },
};
use arcadia_common::error::{Error, Result};
use chrono::Local;
use sqlx::{Postgres, Transaction};
use tokio::sync::broadcast;

impl ConnectionPool {
    pub async fn create_title_group_comment(
        &self,
        title_group_comment: &UserCreatedTitleGroupComment,
        user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<TitleGroupComment> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        let created_title_group_comment = sqlx::query_as!(
            TitleGroupComment,
            r#"
                WITH inserted_comment AS (
                    INSERT INTO title_group_comments (content, title_group_id, created_by_id,
                                                      refers_to_torrent_id, answers_to_comment_id)
                    VALUES ($1, $2, $3, $4, $5)
                    RETURNING id, content, created_at, updated_at, created_by_id, title_group_id, locked, refers_to_torrent_id, answers_to_comment_id
                ),
                updated_user AS (
                    UPDATE users u
                    SET title_group_comments = u.title_group_comments + 1
                    WHERE u.id = (SELECT created_by_id FROM inserted_comment)
                )
                SELECT id, content, created_at, updated_at, created_by_id, title_group_id, locked, refers_to_torrent_id, answers_to_comment_id
                FROM inserted_comment
            "#,
            title_group_comment.content,
            title_group_comment.title_group_id,
            user_id,
            title_group_comment.refers_to_torrent_id,
            title_group_comment.answers_to_comment_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateTitleGroupComment)?;

        let user_ids = Self::notify_users_title_group_comments(
            &mut tx,
            title_group_comment.title_group_id,
            created_title_group_comment.id,
            user_id,
        )
        .await?;

        tx.commit().await?;

        if !user_ids.is_empty() {
            let _ = notification_sender.send(NotificationEvent::TitleGroupComment { user_ids });
        }

        Ok(created_title_group_comment)
    }

    pub async fn find_title_group_comment(&self, comment_id: i64) -> Result<TitleGroupComment> {
        let comment = sqlx::query_as!(
            TitleGroupComment,
            r#"
                SELECT id, content, created_at, updated_at, created_by_id, title_group_id, locked, refers_to_torrent_id, answers_to_comment_id FROM title_group_comments WHERE id = $1
            "#,
            comment_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindTitleGroupComment)?;

        Ok(comment)
    }

    pub async fn update_title_group_comment(
        &self,
        edited_comment: &EditedTitleGroupComment,
        comment_id: i64,
    ) -> Result<TitleGroupComment> {
        let updated_comment = sqlx::query_as!(
            TitleGroupComment,
            r#"
                UPDATE title_group_comments
                SET content = $2, locked = $3, updated_at = NOW()
                WHERE id = $1
                RETURNING id, content, created_at, updated_at, created_by_id, title_group_id, locked, refers_to_torrent_id, answers_to_comment_id
            "#,
            comment_id,
            edited_comment.content,
            edited_comment.locked
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::ErrorWhileUpdatingTitleGroupComment(e.to_string()))?;

        Ok(updated_comment)
    }

    pub async fn delete_title_group_comment(&self, comment_id: i64) -> Result<()> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        let deleted_comment = sqlx::query!(
            r#"
                DELETE FROM title_group_comments WHERE id = $1 RETURNING created_by_id
            "#,
            comment_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteTitleGroupComment)?
        .ok_or(Error::CouldNotFindTitleGroupComment(
            sqlx::Error::RowNotFound,
        ))?;

        sqlx::query!(
            r#"
                UPDATE users SET title_group_comments = title_group_comments - 1 WHERE id = $1
            "#,
            deleted_comment.created_by_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteTitleGroupComment)?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn search_title_group_comments(
        &self,
        form: &TitleGroupCommentSearchQuery,
    ) -> Result<PaginatedResults<TitleGroupCommentSearchResult>> {
        let limit = form.page_size as i64;
        let offset = (form.page - 1) as i64 * form.page_size as i64;

        let results = sqlx::query_as!(
            TitleGroupCommentSearchResult,
            r#"
            SELECT
                c.id,
                c.content,
                c.created_at,
                c.title_group_id,
                tg.name AS title_group_name,
                ROW(u.id, u.username, u.warned, u.banned) AS "created_by!: UserLite"
            FROM title_group_comments c
            JOIN users u ON u.id = c.created_by_id
            JOIN title_groups tg ON tg.id = c.title_group_id
            WHERE $1::TEXT IS NULL OR c.content ILIKE '%' || $1 || '%'
            ORDER BY c.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            form.content,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindTitleGroupComment)?;

        let total_results = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM title_group_comments
            WHERE $1::TEXT IS NULL OR content ILIKE '%' || $1 || '%'
            "#,
            form.content
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindTitleGroupComment)?
        .unwrap_or(0);

        Ok(PaginatedResults {
            results,
            total_items: total_results,
            page: form.page,
            page_size: form.page_size,
        })
    }

    /// Every title group comment written by a user, most recent first.
    pub async fn search_user_title_group_comments(
        &self,
        form: &UserTitleGroupCommentSearchQuery,
    ) -> Result<PaginatedResults<TitleGroupCommentWithLocation>> {
        let limit = form.page_size as i64;
        let offset = (form.page - 1) as i64 * form.page_size as i64;

        let comments = sqlx::query!(
            r#"
            SELECT
                c.id,
                c.content,
                c.created_at,
                c.updated_at,
                c.created_by_id,
                c.title_group_id,
                c.locked,
                c.refers_to_torrent_id,
                c.answers_to_comment_id,
                tg.name AS title_group_name,
                ROW(u.id, u.username, u.class_name, u.banned, u.avatar, u.warned, u.custom_title) AS "created_by!: UserLiteAvatar"
            FROM title_group_comments c
            JOIN users u ON u.id = c.created_by_id
            JOIN title_groups tg ON tg.id = c.title_group_id
            WHERE c.created_by_id = $1
            ORDER BY c.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            form.created_by_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotSearchTitleGroupComments)?;

        let results = comments
            .into_iter()
            .map(|comment| TitleGroupCommentWithLocation {
                comment: TitleGroupCommentHierarchy {
                    id: comment.id,
                    content: comment.content,
                    created_at: comment.created_at.with_timezone(&Local),
                    updated_at: comment.updated_at.with_timezone(&Local),
                    created_by_id: comment.created_by_id,
                    title_group_id: comment.title_group_id,
                    locked: comment.locked,
                    refers_to_torrent_id: comment.refers_to_torrent_id,
                    answers_to_comment_id: comment.answers_to_comment_id,
                    created_by: comment.created_by,
                },
                title_group_name: comment.title_group_name,
            })
            .collect();

        let total_results = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM title_group_comments WHERE created_by_id = $1",
            form.created_by_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotSearchTitleGroupComments)?
        .unwrap_or(0);

        Ok(PaginatedResults {
            results,
            total_items: total_results,
            page: form.page,
            page_size: form.page_size,
        })
    }

    pub async fn find_latest_title_group_comments(
        &self,
        limit: u32,
    ) -> Result<Vec<TitleGroupCommentSearchResult>> {
        let results = sqlx::query_as!(
            TitleGroupCommentSearchResult,
            r#"
            SELECT
                c.id,
                c.content,
                c.created_at,
                c.title_group_id,
                tg.name AS title_group_name,
                ROW(u.id, u.username, u.warned, u.banned) AS "created_by!: UserLite"
            FROM title_group_comments c
            JOIN users u ON u.id = c.created_by_id
            JOIN title_groups tg ON tg.id = c.title_group_id
            ORDER BY c.created_at DESC
            LIMIT $1
            "#,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindTitleGroupComment)?;

        Ok(results)
    }
}
