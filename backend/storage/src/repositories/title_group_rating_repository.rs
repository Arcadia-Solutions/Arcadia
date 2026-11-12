use crate::{
    connection_pool::ConnectionPool,
    models::{
        artist::AffiliatedArtistLite,
        common::PaginatedResults,
        series::SeriesLite,
        title_group::{ContentType, Platform, TitleGroupCategory, TitleGroupHierarchyLite},
        title_group_rating::{
            EditedTitleGroupRating, TitleGroupRating, TitleGroupRatingHierarchy,
            TitleGroupRatingSearchQuery, TitleGroupRatingWithTitleGroup,
            UserCreatedTitleGroupRating, UserTitleGroupRatingSearchQuery,
        },
        user::UserLiteAvatar,
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::{PgConnection, types::Json};

impl ConnectionPool {
    pub async fn create_title_group_rating(
        &self,
        rating: &UserCreatedTitleGroupRating,
        user_id: i32,
    ) -> Result<TitleGroupRating> {
        let mut transaction = self.pool.begin().await?;

        let created_rating = sqlx::query_as!(
            TitleGroupRating,
            r#"
                INSERT INTO title_group_ratings (title_group_id, user_id, rating, comment)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (title_group_id, user_id) DO NOTHING
                RETURNING id, title_group_id, user_id, rating, comment, created_at, updated_at
            "#,
            rating.title_group_id,
            user_id,
            rating.rating,
            rating.comment
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(Error::TitleGroupRatingAlreadyExists)?;

        sqlx::query!(
            "UPDATE users SET title_group_ratings = title_group_ratings + 1 WHERE id = $1",
            user_id
        )
        .execute(&mut *transaction)
        .await?;

        Self::update_title_group_user_rating(&mut transaction, rating.title_group_id).await?;

        // a bare rating isn't worth notifying the subscribers of the title group's comments
        if rating
            .comment
            .as_deref()
            .is_some_and(|text| !text.trim().is_empty())
        {
            Self::notify_users_title_group_comments(
                &mut transaction,
                rating.title_group_id,
                None,
                Some(created_rating.id),
                user_id,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(created_rating)
    }

    pub async fn find_title_group_rating(&self, rating_id: i64) -> Result<TitleGroupRating> {
        sqlx::query_as!(
            TitleGroupRating,
            r#"
                SELECT id, title_group_id, user_id, rating, comment, created_at, updated_at
                FROM title_group_ratings
                WHERE id = $1
            "#,
            rating_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(Error::TitleGroupRatingNotFound)
    }

    pub async fn update_title_group_rating(
        &self,
        edited_rating: &EditedTitleGroupRating,
        rating_id: i64,
    ) -> Result<TitleGroupRating> {
        let mut transaction = self.pool.begin().await?;

        let updated_rating = sqlx::query_as!(
            TitleGroupRating,
            r#"
                UPDATE title_group_ratings
                SET rating = $2, comment = $3, updated_at = NOW()
                WHERE id = $1
                RETURNING id, title_group_id, user_id, rating, comment, created_at, updated_at
            "#,
            rating_id,
            edited_rating.rating,
            edited_rating.comment
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(Error::TitleGroupRatingNotFound)?;

        Self::update_title_group_user_rating(&mut transaction, updated_rating.title_group_id)
            .await?;

        transaction.commit().await?;

        Ok(updated_rating)
    }

    pub async fn delete_title_group_rating(&self, rating_id: i64) -> Result<()> {
        let mut transaction = self.pool.begin().await?;

        let deleted_rating = sqlx::query!(
            "DELETE FROM title_group_ratings WHERE id = $1 RETURNING user_id, title_group_id",
            rating_id
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(Error::TitleGroupRatingNotFound)?;

        sqlx::query!(
            "UPDATE users SET title_group_ratings = title_group_ratings - 1 WHERE id = $1",
            deleted_rating.user_id
        )
        .execute(&mut *transaction)
        .await?;

        Self::update_title_group_user_rating(&mut transaction, deleted_rating.title_group_id)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    /// Recomputes the cached average and amount of ratings of a title group.
    pub async fn update_title_group_user_rating(
        connection: &mut PgConnection,
        title_group_id: i32,
    ) -> Result<()> {
        // Concurrent transactions rating the same title group wait here for each other, so
        // the recompute below runs with a fresh snapshot that includes their committed ratings
        sqlx::query!(
            "SELECT id FROM title_groups WHERE id = $1 FOR UPDATE",
            title_group_id
        )
        .fetch_optional(&mut *connection)
        .await?;

        sqlx::query!(
            r#"
                UPDATE title_groups
                SET (user_rating_average, user_rating_count) = (
                    SELECT AVG(rating)::REAL, COUNT(*)::INT
                    FROM title_group_ratings
                    WHERE title_group_id = $1
                )
                WHERE id = $1
            "#,
            title_group_id
        )
        .execute(connection)
        .await?;

        Ok(())
    }

    /// Whether the user completed the download of, or was seen seeding, a torrent of the title
    /// group.
    pub async fn has_snatched_or_seeded_title_group(
        &self,
        user_id: i32,
        title_group_id: i32,
    ) -> Result<bool> {
        Ok(sqlx::query_scalar!(
            r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM torrent_activities ta
                    JOIN torrents t ON t.id = ta.torrent_id
                    JOIN edition_groups eg ON eg.id = t.edition_group_id
                    WHERE eg.title_group_id = $2
                      AND ta.user_id = $1
                      AND (ta.completed_at IS NOT NULL OR ta.first_seen_seeding_at IS NOT NULL)
                ) AS "exists!"
            "#,
            user_id,
            title_group_id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn find_user_title_group_rating(
        &self,
        user_id: i32,
        title_group_id: i32,
    ) -> Result<Option<TitleGroupRating>> {
        Ok(sqlx::query_as!(
            TitleGroupRating,
            r#"
                SELECT id, title_group_id, user_id, rating, comment, created_at, updated_at
                FROM title_group_ratings
                WHERE user_id = $1 AND title_group_id = $2
            "#,
            user_id,
            title_group_id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// The ratings of a title group matching the query, ties broken by the most recent first.
    pub async fn search_title_group_ratings(
        &self,
        form: &TitleGroupRatingSearchQuery,
    ) -> Result<PaginatedResults<TitleGroupRatingHierarchy>> {
        let limit = form.page_size as i64;
        let offset = (form.page as i64 - 1).max(0) * limit;

        let results = sqlx::query_as!(
            TitleGroupRatingHierarchy,
            r#"
                SELECT
                    r.id, r.rating, r.comment, r.created_at, r.updated_at,
                    ROW(u.id, u.username, u.class_name, u.banned, u.avatar, u.warned, u.custom_title) AS "created_by!: UserLiteAvatar"
                FROM title_group_ratings r
                JOIN users u ON u.id = r.user_id
                WHERE r.title_group_id = $1
                  AND r.rating BETWEEN $2 AND $3
                  AND COALESCE(r.comment, '') ILIKE '%' || $4 || '%'
                ORDER BY
                    CASE WHEN $7 = 'created_at' AND $8 = 'asc' THEN r.created_at END ASC,
                    CASE WHEN $7 = 'created_at' AND $8 = 'desc' THEN r.created_at END DESC,
                    CASE WHEN $7 = 'rating' AND $8 = 'asc' THEN r.rating END ASC,
                    CASE WHEN $7 = 'rating' AND $8 = 'desc' THEN r.rating END DESC,
                    r.id DESC
                LIMIT $5 OFFSET $6
            "#,
            form.title_group_id,
            form.minimum_rating,
            form.maximum_rating,
            form.content,
            limit,
            offset,
            form.order_by_column.to_string(),
            form.order_by_direction.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        let total_items = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) AS "count!"
                FROM title_group_ratings
                WHERE title_group_id = $1
                  AND rating BETWEEN $2 AND $3
                  AND COALESCE(comment, '') ILIKE '%' || $4 || '%'
            "#,
            form.title_group_id,
            form.minimum_rating,
            form.maximum_rating,
            form.content
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            total_items,
            page: form.page,
            page_size: form.page_size,
        })
    }

    /// The ratings written by a user matching the query, ties broken by the most recent first.
    pub async fn search_user_title_group_ratings(
        &self,
        form: &UserTitleGroupRatingSearchQuery,
    ) -> Result<PaginatedResults<TitleGroupRatingWithTitleGroup>> {
        let limit = form.page_size as i64;
        let offset = (form.page as i64 - 1).max(0) * limit;

        let rows = sqlx::query!(
            r#"
                SELECT
                    r.id, r.user_id, r.rating, r.comment, r.created_at, r.updated_at,
                    tg.id AS title_group_id,
                    tg.name AS title_group_name,
                    tg.covers,
                    tg.category AS "category: TitleGroupCategory",
                    tg.content_type AS "content_type: ContentType",
                    tg.original_release_date,
                    tg.original_release_date_only_year_known,
                    tg.platform AS "platform: Platform",
                    tg.user_rating_average,
                    tg.user_rating_count,
                    COALESCE((
                        SELECT ARRAY_AGG(tag.name ORDER BY tag.name)
                        FROM title_group_applied_tags applied_tag
                        JOIN title_group_tags tag ON tag.id = applied_tag.tag_id
                        WHERE applied_tag.title_group_id = tg.id
                    ), '{}') AS "tags!",
                    COALESCE((
                        SELECT jsonb_agg(jsonb_build_object('artist_id', a.id, 'name', a.name))
                        FROM affiliated_artists aa
                        JOIN artists a ON a.id = aa.artist_id
                        WHERE aa.title_group_id = tg.id
                    ), '[]'::jsonb) AS "affiliated_artists!: Json<Vec<AffiliatedArtistLite>>",
                    CASE WHEN s.id IS NOT NULL
                        THEN jsonb_build_object('id', s.id, 'name', s.name)
                    END AS "series: Json<SeriesLite>"
                FROM title_group_ratings r
                JOIN title_groups tg ON tg.id = r.title_group_id
                LEFT JOIN series s ON s.id = tg.series_id
                WHERE r.user_id = $1
                  AND r.rating BETWEEN $2 AND $3
                  AND COALESCE(r.comment, '') ILIKE '%' || $4 || '%'
                ORDER BY
                    CASE WHEN $7 = 'created_at' AND $8 = 'asc' THEN r.created_at END ASC,
                    CASE WHEN $7 = 'created_at' AND $8 = 'desc' THEN r.created_at END DESC,
                    CASE WHEN $7 = 'rating' AND $8 = 'asc' THEN r.rating END ASC,
                    CASE WHEN $7 = 'rating' AND $8 = 'desc' THEN r.rating END DESC,
                    r.id DESC
                LIMIT $5 OFFSET $6
            "#,
            form.user_id,
            form.minimum_rating,
            form.maximum_rating,
            form.content,
            limit,
            offset,
            form.order_by_column.to_string(),
            form.order_by_direction.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| TitleGroupRatingWithTitleGroup {
                id: row.id,
                user_id: row.user_id,
                rating: row.rating,
                comment: row.comment,
                created_at: row.created_at,
                updated_at: row.updated_at,
                title_group: TitleGroupHierarchyLite {
                    id: row.title_group_id,
                    name: row.title_group_name,
                    covers: row.covers,
                    category: row.category,
                    content_type: row.content_type,
                    tags: row.tags,
                    original_release_date: row.original_release_date,
                    original_release_date_only_year_known: row
                        .original_release_date_only_year_known,
                    platform: row.platform,
                    edition_groups: Json(vec![]),
                    affiliated_artists: row.affiliated_artists,
                    series: row.series,
                    user_rating_average: row.user_rating_average,
                    user_rating_count: row.user_rating_count,
                },
            })
            .collect();

        let total_items = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) AS "count!"
                FROM title_group_ratings
                WHERE user_id = $1
                  AND rating BETWEEN $2 AND $3
                  AND COALESCE(comment, '') ILIKE '%' || $4 || '%'
            "#,
            form.user_id,
            form.minimum_rating,
            form.maximum_rating,
            form.content
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            total_items,
            page: form.page,
            page_size: form.page_size,
        })
    }
}
