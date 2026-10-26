use crate::{
    connection_pool::ConnectionPool,
    models::title_group_bookmark::{
        EditedTitleGroupBookmark, TitleGroupBookmark, UserCreatedTitleGroupBookmark,
    },
};
use arcadia_common::error::{Error, Result};

impl ConnectionPool {
    pub async fn create_title_group_bookmark(
        &self,
        bookmark: &UserCreatedTitleGroupBookmark,
        current_user_id: i32,
    ) -> Result<TitleGroupBookmark> {
        let created_bookmark = sqlx::query_as!(
            TitleGroupBookmark,
            r#"
                INSERT INTO title_group_bookmarks (user_id, title_group_id, description)
                VALUES ($1, $2, $3)
                RETURNING id, created_at, user_id, title_group_id, description
            "#,
            current_user_id,
            bookmark.title_group_id,
            bookmark.description,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateTitleGroupBookmark)?;

        Ok(created_bookmark)
    }

    pub async fn find_title_group_bookmark(
        &self,
        bookmark_id: i64,
        current_user_id: i32,
    ) -> Result<TitleGroupBookmark> {
        let bookmark = sqlx::query_as!(
            TitleGroupBookmark,
            r#"
            SELECT
                id, created_at, user_id, title_group_id, description
            FROM title_group_bookmarks
            WHERE id = $1 AND user_id = $2
            "#,
            bookmark_id,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindTitleGroupBookmark)?;

        Ok(bookmark)
    }

    pub async fn update_title_group_bookmark(
        &self,
        edited_bookmark: &EditedTitleGroupBookmark,
        bookmark_id: i64,
        current_user_id: i32,
    ) -> Result<TitleGroupBookmark> {
        let updated_bookmark = sqlx::query_as!(
            TitleGroupBookmark,
            r#"
            UPDATE title_group_bookmarks
            SET
                description = $3
            WHERE id = $1 AND user_id = $2
            RETURNING
                id, created_at, user_id, title_group_id, description
            "#,
            bookmark_id,
            current_user_id,
            edited_bookmark.description
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::ErrorWhileUpdatingTitleGroupBookmark(e.to_string()))?;

        Ok(updated_bookmark)
    }

    pub async fn delete_title_group_bookmark(
        &self,
        bookmark_id: i64,
        current_user_id: i32,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
                DELETE FROM title_group_bookmarks
                WHERE id = $1 AND user_id = $2;
            "#,
        )
        .bind(bookmark_id)
        .bind(current_user_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::CouldNotFindTitleGroupBookmark(
                sqlx::Error::RowNotFound,
            ));
        }

        Ok(())
    }
}
