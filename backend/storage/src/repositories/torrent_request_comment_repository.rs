use crate::{
    connection_pool::ConnectionPool,
    models::{
        common::PaginatedResults,
        notification::NotificationEvent,
        torrent_request_comment::{
            TorrentRequestComment, TorrentRequestCommentHierarchy,
            TorrentRequestCommentWithLocation, UserTorrentRequestCommentSearchQuery,
        },
        user::UserLiteAvatar,
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::{Postgres, Transaction};
use tokio::sync::broadcast;

impl ConnectionPool {
    pub async fn create_torrent_request_comment(
        &self,
        torrent_request_id: i64,
        user_id: i32,
        content: &str,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<TorrentRequestComment> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        let created_torrent_request_comment = sqlx::query_as!(
            TorrentRequestComment,
            r#"
                WITH inserted_comment AS (
                    INSERT INTO torrent_request_comments (torrent_request_id, created_by_id, content)
                    VALUES ($1, $2, $3)
                    RETURNING id, torrent_request_id, created_by_id, content, created_at, updated_at
                ),
                updated_user AS (
                    UPDATE users u
                    SET request_comments = u.request_comments + 1
                    WHERE u.id = (SELECT created_by_id FROM inserted_comment)
                )
                SELECT
                    inserted_comment.id,
                    inserted_comment.torrent_request_id,
                    inserted_comment.created_by_id,
                    inserted_comment.content,
                    inserted_comment.created_at,
                    inserted_comment.updated_at
                FROM inserted_comment
            "#,
            torrent_request_id,
            user_id,
            content
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateTorrentRequestComment)?;

        let user_ids = Self::notify_users_torrent_request_comments(
            &mut tx,
            torrent_request_id,
            created_torrent_request_comment.id,
            user_id,
        )
        .await?;

        tx.commit().await?;

        if !user_ids.is_empty() {
            let _ = notification_sender.send(NotificationEvent::TorrentRequestComment { user_ids });
        }

        Ok(created_torrent_request_comment)
    }

    pub async fn delete_torrent_request_comment(&self, comment_id: i64) -> Result<()> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        let deleted_comment = sqlx::query!(
            r#"
                DELETE FROM torrent_request_comments WHERE id = $1 RETURNING created_by_id
            "#,
            comment_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteTorrentRequestComment)?
        .ok_or(Error::CouldNotFindTorrentRequestComment(
            sqlx::Error::RowNotFound,
        ))?;

        sqlx::query!(
            r#"
                UPDATE users SET request_comments = request_comments - 1 WHERE id = $1
            "#,
            deleted_comment.created_by_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteTorrentRequestComment)?;

        tx.commit().await?;

        Ok(())
    }

    /// Every torrent request comment written by a user, most recent first.
    pub async fn search_user_torrent_request_comments(
        &self,
        form: &UserTorrentRequestCommentSearchQuery,
    ) -> Result<PaginatedResults<TorrentRequestCommentWithLocation>> {
        let limit = form.page_size as i64;
        let offset = (form.page - 1) as i64 * form.page_size as i64;

        let comments = sqlx::query!(
            r#"
            SELECT
                c.id,
                c.torrent_request_id,
                c.content,
                c.created_at,
                c.updated_at,
                c.created_by_id,
                tr.title_group_id,
                tg.name AS title_group_name,
                ROW(u.id, u.username, u.class_name, u.banned, u.avatar, u.warned, u.custom_title) AS "created_by!: UserLiteAvatar"
            FROM torrent_request_comments c
            JOIN users u ON u.id = c.created_by_id
            JOIN torrent_requests tr ON tr.id = c.torrent_request_id
            JOIN title_groups tg ON tg.id = tr.title_group_id
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
        .map_err(Error::CouldNotSearchTorrentRequestComments)?;

        let results = comments
            .into_iter()
            .map(|comment| TorrentRequestCommentWithLocation {
                comment: TorrentRequestCommentHierarchy {
                    id: comment.id,
                    torrent_request_id: comment.torrent_request_id,
                    created_by_id: comment.created_by_id,
                    content: comment.content,
                    created_at: comment.created_at,
                    updated_at: comment.updated_at,
                    created_by: comment.created_by,
                },
                title_group_id: comment.title_group_id,
                title_group_name: comment.title_group_name,
            })
            .collect();

        let total_results = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM torrent_request_comments WHERE created_by_id = $1",
            form.created_by_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotSearchTorrentRequestComments)?
        .unwrap_or(0);

        Ok(PaginatedResults {
            results,
            total_items: total_results,
            page: form.page,
            page_size: form.page_size,
        })
    }
}
