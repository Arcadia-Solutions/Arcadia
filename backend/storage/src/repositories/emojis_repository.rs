use crate::{
    connection_pool::ConnectionPool,
    models::emoji::{Emoji, EmojiImage, EmojiUsage, ReorderEmojis},
};
use arcadia_common::error::{Error, Result};

fn map_emoji_write_error(error: sqlx::Error) -> Error {
    if let sqlx::Error::Database(db_err) = &error
        && db_err.code().as_deref() == Some("23505")
        && db_err.constraint() == Some("emojis_name_key")
    {
        return Error::EmojiNameAlreadyExists;
    }
    Error::from(error)
}

impl ConnectionPool {
    pub async fn find_emojis(&self) -> Result<Vec<Emoji>> {
        let emojis = sqlx::query_as!(
            Emoji,
            r#"
            SELECT
                id,
                name,
                unicode_character,
                EXTRACT(EPOCH FROM updated_at)::BIGINT AS "image_version!",
                sort_order,
                enabled
            FROM emojis
            ORDER BY sort_order, id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(emojis)
    }

    pub async fn find_emojis_usage(&self) -> Result<Vec<EmojiUsage>> {
        let usage = sqlx::query_as!(
            EmojiUsage,
            r#"
            SELECT e.id AS "emoji_id!", COUNT(fpr.emoji_id) AS "reactions_amount!"
            FROM emojis e
            LEFT JOIN forum_post_reactions fpr ON fpr.emoji_id = e.id
            GROUP BY e.id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(usage)
    }

    pub async fn find_emoji_image(&self, emoji_id: i32) -> Result<EmojiImage> {
        sqlx::query_as!(
            EmojiImage,
            r#"
            SELECT
                image AS "image!",
                image_mime_type AS "image_mime_type!",
                EXTRACT(EPOCH FROM updated_at)::BIGINT AS "image_version!"
            FROM emojis
            WHERE id = $1 AND image IS NOT NULL
            "#,
            emoji_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(Error::EmojiNotFound)
    }

    pub async fn create_emoji(
        &self,
        name: &str,
        unicode_character: Option<&str>,
        image: Option<&[u8]>,
        image_mime_type: Option<&str>,
    ) -> Result<Emoji> {
        let emoji = sqlx::query_as!(
            Emoji,
            r#"
            INSERT INTO emojis (name, sort_order, unicode_character, image, image_mime_type)
            VALUES (
                $1,
                COALESCE((SELECT MAX(sort_order) FROM emojis), 0) + 1,
                $2,
                $3,
                $4
            )
            RETURNING
                id,
                name,
                unicode_character,
                EXTRACT(EPOCH FROM updated_at)::BIGINT AS "image_version!",
                sort_order,
                enabled
            "#,
            name,
            unicode_character,
            image,
            image_mime_type
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_emoji_write_error)?;

        Ok(emoji)
    }

    /// Updates an emoji. When `image` is `None` and `unicode_character` is `None`, the current
    /// representation is kept and only the name changes. The sort order is not editable here,
    /// it only changes through `reorder_emojis`.
    pub async fn update_emoji(
        &self,
        emoji_id: i32,
        name: &str,
        unicode_character: Option<&str>,
        image: Option<&[u8]>,
        image_mime_type: Option<&str>,
    ) -> Result<Emoji> {
        let representation_changed = unicode_character.is_some() || image.is_some();

        sqlx::query_as!(
            Emoji,
            r#"
            UPDATE emojis
            SET name = $2,
                unicode_character = CASE WHEN $5 THEN $3 ELSE unicode_character END,
                image = CASE WHEN $5 THEN $4 ELSE image END,
                image_mime_type = CASE WHEN $5 THEN $6 ELSE image_mime_type END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id,
                name,
                unicode_character,
                EXTRACT(EPOCH FROM updated_at)::BIGINT AS "image_version!",
                sort_order,
                enabled
            "#,
            emoji_id,
            name,
            unicode_character,
            image,
            representation_changed,
            image_mime_type
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_emoji_write_error)?
        .ok_or(Error::EmojiNotFound)
    }

    pub async fn delete_emoji(&self, emoji_id: i32) -> Result<()> {
        let deleted = sqlx::query!(r#"DELETE FROM emojis WHERE id = $1"#, emoji_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if deleted == 0 {
            return Err(Error::EmojiNotFound);
        }

        Ok(())
    }

    /// Reorders emojis. Runs inside a transaction so a body naming an unknown emoji id rolls
    /// back the whole bulk update instead of leaving the known ids reordered while the caller
    /// is told the request failed.
    pub async fn reorder_emojis(&self, reorder: &ReorderEmojis) -> Result<()> {
        let ids: Vec<i32> = reorder.emojis.iter().map(|entry| entry.id).collect();
        let sort_orders: Vec<i16> = reorder
            .emojis
            .iter()
            .map(|entry| entry.sort_order)
            .collect();

        let mut tx = self.pool.begin().await?;

        let result = sqlx::query!(
            r#"
                UPDATE emojis
                SET sort_order = new_values.sort_order
                FROM UNNEST($1::int[], $2::smallint[]) AS new_values(id, sort_order)
                WHERE emojis.id = new_values.id
            "#,
            &ids,
            &sort_orders
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotReorderEmojis)?;

        if result.rows_affected() != ids.len() as u64 {
            // Dropping the transaction without committing rolls back the partial update.
            return Err(Error::EmojiNotFound);
        }

        tx.commit().await?;

        Ok(())
    }

    /// Enables or disables an emoji. A disabled emoji is hidden from the reaction picker and
    /// can no longer be reacted with, but existing reactions using it are kept.
    pub async fn set_emoji_enabled(&self, emoji_id: i32, enabled: bool) -> Result<()> {
        let updated = sqlx::query!(
            r#"UPDATE emojis SET enabled = $2 WHERE id = $1"#,
            emoji_id,
            enabled
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if updated == 0 {
            return Err(Error::EmojiNotFound);
        }

        Ok(())
    }
}
