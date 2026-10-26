use crate::{
    connection_pool::ConnectionPool,
    models::site_highlight::{
        CreateSiteHighlight, EditSiteHighlight, SiteHighlight, SiteHighlightForHome,
        SiteHighlightItemType,
    },
};
use arcadia_common::error::{Error, Result};

fn map_site_highlight_write_error(error: sqlx::Error, fallback: fn(sqlx::Error) -> Error) -> Error {
    if let sqlx::Error::Database(db_err) = &error
        && db_err.code().as_deref() == Some("23505")
        && db_err.constraint() == Some("site_highlights_position_key")
    {
        return Error::SiteHighlightPositionTaken;
    }
    fallback(error)
}

fn split_item_id(
    item_type: SiteHighlightItemType,
    item_id: i64,
) -> (Option<i32>, Option<i64>, Option<i64>) {
    match item_type {
        SiteHighlightItemType::TitleGroup => (Some(item_id as i32), None, None),
        SiteHighlightItemType::Series => (None, Some(item_id), None),
        SiteHighlightItemType::Artist => (None, None, Some(item_id)),
    }
}

impl ConnectionPool {
    pub async fn find_all_site_highlights(&self) -> Result<Vec<SiteHighlight>> {
        let highlights = sqlx::query_as!(
            SiteHighlight,
            r#"
                SELECT
                    id,
                    created_at,
                    created_by_id,
                    alias,
                    item_type as "item_type: SiteHighlightItemType",
                    COALESCE(title_group_id::BIGINT, series_id, artist_id) as "item_id!",
                    forum_thread_id,
                    enabled,
                    position
                FROM site_highlights
                ORDER BY position ASC, id ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotGetSiteHighlights)?;

        Ok(highlights)
    }

    pub async fn find_enabled_site_highlights_for_home(&self) -> Result<Vec<SiteHighlightForHome>> {
        let highlights = sqlx::query_as!(
            SiteHighlightForHome,
            r#"
                SELECT
                    sh.id,
                    sh.alias,
                    sh.item_type as "item_type: SiteHighlightItemType",
                    COALESCE(sh.title_group_id::BIGINT, sh.series_id, sh.artist_id) as "item_id!",
                    CASE sh.item_type
                        WHEN 'title_group' THEN (SELECT covers[1] FROM title_groups WHERE id = sh.title_group_id)
                        WHEN 'series'      THEN (SELECT covers[1] FROM series       WHERE id = sh.series_id)
                        WHEN 'artist'      THEN (SELECT pictures[1] FROM artists    WHERE id = sh.artist_id)
                    END AS item_image,
                    sh.forum_thread_id,
                    sh.position
                FROM site_highlights sh
                WHERE sh.enabled = TRUE
                ORDER BY sh.position ASC, sh.id ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotGetSiteHighlights)?;

        Ok(highlights)
    }

    pub async fn create_site_highlight(
        &self,
        payload: &CreateSiteHighlight,
        user_id: i32,
    ) -> Result<SiteHighlight> {
        let mut tx = self.pool.begin().await?;

        let (title_group_id, series_id, artist_id) =
            split_item_id(payload.item_type, payload.item_id);

        let inserted = sqlx::query_as!(
            SiteHighlight,
            r#"
                INSERT INTO site_highlights (
                    created_by_id, alias, item_type, title_group_id, series_id, artist_id,
                    forum_thread_id, enabled, position
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING
                    id,
                    created_at,
                    created_by_id,
                    alias,
                    item_type as "item_type: SiteHighlightItemType",
                    COALESCE(title_group_id::BIGINT, series_id, artist_id) as "item_id!",
                    forum_thread_id,
                    enabled,
                    position
            "#,
            user_id,
            payload.alias,
            payload.item_type as SiteHighlightItemType,
            title_group_id,
            series_id,
            artist_id,
            payload.forum_thread_id,
            payload.enabled,
            payload.position,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| map_site_highlight_write_error(e, Error::CouldNotCreateSiteHighlight))?;

        Self::insert_related_forum_thread_tx(
            &mut tx,
            payload.item_type,
            payload.item_id,
            payload.forum_thread_id,
            user_id,
        )
        .await?;

        tx.commit().await?;

        Ok(inserted)
    }

    pub async fn edit_site_highlight(
        &self,
        id: i32,
        payload: &EditSiteHighlight,
    ) -> Result<SiteHighlight> {
        let mut tx = self.pool.begin().await?;

        let existing = sqlx::query_as!(
            SiteHighlight,
            r#"
                SELECT
                    id,
                    created_at,
                    created_by_id,
                    alias,
                    item_type as "item_type: SiteHighlightItemType",
                    COALESCE(title_group_id::BIGINT, series_id, artist_id) as "item_id!",
                    forum_thread_id,
                    enabled,
                    position
                FROM site_highlights
                WHERE id = $1
                FOR UPDATE
            "#,
            id,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(Error::CouldNotEditSiteHighlight)?
        .ok_or(Error::SiteHighlightNotFound)?;

        let new_item_type = payload.item_type.unwrap_or(existing.item_type);
        let new_item_id = payload.item_id.unwrap_or(existing.item_id);
        let new_forum_thread_id = payload.forum_thread_id.unwrap_or(existing.forum_thread_id);

        let (new_title_group_id, new_series_id, new_artist_id) =
            split_item_id(new_item_type, new_item_id);

        let edited = sqlx::query_as!(
            SiteHighlight,
            r#"
                UPDATE site_highlights
                SET
                    alias           = COALESCE($2, alias),
                    item_type       = $3,
                    title_group_id  = $4,
                    series_id       = $5,
                    artist_id       = $6,
                    forum_thread_id = $7,
                    enabled         = COALESCE($8, enabled),
                    position        = COALESCE($9, position)
                WHERE id = $1
                RETURNING
                    id,
                    created_at,
                    created_by_id,
                    alias,
                    item_type as "item_type: SiteHighlightItemType",
                    COALESCE(title_group_id::BIGINT, series_id, artist_id) as "item_id!",
                    forum_thread_id,
                    enabled,
                    position
            "#,
            id,
            payload.alias,
            new_item_type as SiteHighlightItemType,
            new_title_group_id,
            new_series_id,
            new_artist_id,
            new_forum_thread_id,
            payload.enabled,
            payload.position,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| map_site_highlight_write_error(e, Error::CouldNotEditSiteHighlight))?;

        let key_changed = existing.item_type != new_item_type
            || existing.item_id != new_item_id
            || existing.forum_thread_id != new_forum_thread_id;

        if key_changed {
            if payload.remove_previous_related_thread {
                Self::delete_related_forum_thread_tx(
                    &mut tx,
                    existing.item_type,
                    existing.item_id,
                    existing.forum_thread_id,
                )
                .await?;
            }
            Self::insert_related_forum_thread_tx(
                &mut tx,
                new_item_type,
                new_item_id,
                new_forum_thread_id,
                existing.created_by_id,
            )
            .await?;
        }

        tx.commit().await?;

        Ok(edited)
    }

    pub async fn delete_site_highlight(&self, id: i32) -> Result<()> {
        let result = sqlx::query!("DELETE FROM site_highlights WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(Error::CouldNotDeleteSiteHighlight)?;

        if result.rows_affected() == 0 {
            return Err(Error::SiteHighlightNotFound);
        }
        Ok(())
    }
}
