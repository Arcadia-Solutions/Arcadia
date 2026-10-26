use crate::{
    connection_pool::ConnectionPool,
    models::{
        reaction::{ContentReaction, ForumPostReactionUsers},
        user::UserLite,
    },
};
use arcadia_common::error::{Error, Result};
use std::collections::HashMap;

fn map_reaction_write_error(error: sqlx::Error) -> Error {
    if let sqlx::Error::Database(db_err) = &error
        && db_err.code().as_deref() == Some("23503")
        && matches!(
            db_err.constraint(),
            Some("forum_post_reactions_emoji_id_fkey")
                | Some("forum_post_reactions_forum_post_id_fkey")
        )
    {
        return Error::EmojiOrForumPostNotFound;
    }
    Error::from(error)
}

impl ConnectionPool {
    /// Reacting with an emoji that does not exist yields the same not found error as reacting
    /// with an emoji that has been disabled since the picker was loaded, but a disabled emoji
    /// gets its own clear error so the caller can tell the two apart.
    pub async fn create_forum_post_reaction(
        &self,
        forum_post_id: i64,
        emoji_id: i32,
        user_id: i32,
    ) -> Result<()> {
        let emoji_enabled = sqlx::query_scalar!(
            r#"
            WITH target_emoji AS (
                SELECT id, enabled FROM emojis WHERE id = $3
            ), inserted_reaction AS (
                INSERT INTO forum_post_reactions (forum_post_id, user_id, emoji_id)
                SELECT $1, $2, target_emoji.id FROM target_emoji WHERE target_emoji.enabled
                ON CONFLICT DO NOTHING
                RETURNING emoji_id
            )
            SELECT enabled AS "enabled!" FROM target_emoji
            "#,
            forum_post_id,
            user_id,
            emoji_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_reaction_write_error)?;

        match emoji_enabled {
            None => Err(Error::EmojiOrForumPostNotFound),
            Some(false) => Err(Error::EmojiDisabled),
            Some(true) => Ok(()),
        }
    }

    /// Removing a reaction the user never left is reported as not found rather than silently
    /// succeeding, so a client sending the wrong post or emoji id sees it.
    pub async fn delete_forum_post_reaction(
        &self,
        forum_post_id: i64,
        emoji_id: i32,
        user_id: i32,
    ) -> Result<()> {
        let deleted = sqlx::query!(
            r#"
            DELETE FROM forum_post_reactions
            WHERE forum_post_id = $1 AND user_id = $2 AND emoji_id = $3
            "#,
            forum_post_id,
            user_id,
            emoji_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if deleted == 0 {
            return Err(Error::EmojiOrForumPostNotFound);
        }

        Ok(())
    }

    /// Reactions of a whole page of forum posts, in one query, keyed by forum post id. Posts
    /// without any reaction are absent from the map.
    pub async fn find_reactions_for_forum_posts(
        &self,
        forum_post_ids: &[i64],
        user_id: i32,
    ) -> Result<HashMap<i64, Vec<ContentReaction>>> {
        if forum_post_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = sqlx::query!(
            r#"
            SELECT
                fpr.forum_post_id,
                fpr.emoji_id,
                e.name AS emoji_name,
                e.unicode_character AS emoji_unicode_character,
                EXTRACT(EPOCH FROM e.updated_at)::BIGINT AS "emoji_image_version!",
                COUNT(*) AS "amount!",
                BOOL_OR(fpr.user_id = $2) AS "reacted_by_current_user!"
            FROM forum_post_reactions fpr
            JOIN emojis e ON e.id = fpr.emoji_id
            WHERE fpr.forum_post_id = ANY($1)
            GROUP BY fpr.forum_post_id, fpr.emoji_id, e.id, e.name, e.unicode_character,
                     e.updated_at, e.sort_order
            ORDER BY e.sort_order, e.id
            "#,
            forum_post_ids,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut reactions_per_post: HashMap<i64, Vec<ContentReaction>> = HashMap::new();
        for row in rows {
            reactions_per_post
                .entry(row.forum_post_id)
                .or_default()
                .push(ContentReaction {
                    emoji_id: row.emoji_id,
                    emoji_name: row.emoji_name,
                    emoji_unicode_character: row.emoji_unicode_character,
                    emoji_image_version: row.emoji_image_version,
                    amount: row.amount,
                    reacted_by_current_user: row.reacted_by_current_user,
                });
        }

        Ok(reactions_per_post)
    }

    /// Users who reacted to a forum post, grouped by emoji, oldest reaction first, capped at
    /// 100 users per emoji. `total_amount` always reports the real amount.
    pub async fn find_forum_post_reaction_users(
        &self,
        forum_post_id: i64,
    ) -> Result<Vec<ForumPostReactionUsers>> {
        let rows = sqlx::query!(
            r#"
            SELECT emoji_id, user_id, username, warned, banned, total_amount AS "total_amount!"
            FROM (
                SELECT
                    fpr.emoji_id,
                    u.id AS user_id,
                    u.username,
                    u.warned,
                    u.banned,
                    e.sort_order,
                    ROW_NUMBER() OVER (
                        PARTITION BY fpr.emoji_id ORDER BY fpr.created_at, fpr.user_id
                    ) AS row_number,
                    COUNT(*) OVER (PARTITION BY fpr.emoji_id) AS total_amount
                FROM forum_post_reactions fpr
                JOIN users u ON u.id = fpr.user_id
                JOIN emojis e ON e.id = fpr.emoji_id
                WHERE fpr.forum_post_id = $1
            ) ranked_reactions
            WHERE row_number <= 100
            ORDER BY sort_order, emoji_id, row_number
            "#,
            forum_post_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut grouped: Vec<ForumPostReactionUsers> = Vec::new();
        for row in rows {
            let user = UserLite {
                id: row.user_id,
                username: row.username,
                warned: row.warned,
                banned: row.banned,
            };
            match grouped.last_mut() {
                Some(group) if group.emoji_id == row.emoji_id => group.users.push(user),
                _ => grouped.push(ForumPostReactionUsers {
                    emoji_id: row.emoji_id,
                    users: vec![user],
                    total_amount: row.total_amount,
                }),
            }
        }

        Ok(grouped)
    }
}
