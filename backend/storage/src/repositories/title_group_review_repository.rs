use crate::{
    connection_pool::ConnectionPool,
    models::{
        common::PaginatedResults,
        title_group_review::{
            EditedTitleGroupReview, TitleGroupReview, TitleGroupReviewHierarchy,
            TitleGroupReviewSearchQuery, TitleGroupReviewWithTitleGroup,
            UserCreatedTitleGroupReview, UserTitleGroupReviewSearchQuery,
        },
        user::UserLiteAvatar,
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::PgConnection;

impl ConnectionPool {
    pub async fn create_title_group_review(
        &self,
        review: &UserCreatedTitleGroupReview,
        user_id: i32,
    ) -> Result<TitleGroupReview> {
        let mut transaction = self.pool.begin().await?;

        let created_review = sqlx::query_as!(
            TitleGroupReview,
            r#"
                INSERT INTO title_group_reviews (title_group_id, user_id, rating, review)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (title_group_id, user_id) DO NOTHING
                RETURNING id, title_group_id, user_id, rating, review, created_at, updated_at
            "#,
            review.title_group_id,
            user_id,
            review.rating,
            review.review
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(Error::TitleGroupReviewAlreadyExists)?;

        sqlx::query!(
            "UPDATE users SET title_group_reviews = title_group_reviews + 1 WHERE id = $1",
            user_id
        )
        .execute(&mut *transaction)
        .await?;

        Self::update_title_group_user_rating(&mut transaction, review.title_group_id).await?;

        // a bare rating isn't worth notifying the subscribers of the title group's comments
        if review
            .review
            .as_deref()
            .is_some_and(|text| !text.trim().is_empty())
        {
            Self::notify_users_title_group_comments(
                &mut transaction,
                review.title_group_id,
                None,
                Some(created_review.id),
                user_id,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(created_review)
    }

    pub async fn find_title_group_review(&self, review_id: i64) -> Result<TitleGroupReview> {
        sqlx::query_as!(
            TitleGroupReview,
            r#"
                SELECT id, title_group_id, user_id, rating, review, created_at, updated_at
                FROM title_group_reviews
                WHERE id = $1
            "#,
            review_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(Error::TitleGroupReviewNotFound)
    }

    pub async fn update_title_group_review(
        &self,
        edited_review: &EditedTitleGroupReview,
        review_id: i64,
    ) -> Result<TitleGroupReview> {
        let mut transaction = self.pool.begin().await?;

        let updated_review = sqlx::query_as!(
            TitleGroupReview,
            r#"
                UPDATE title_group_reviews
                SET rating = $2, review = $3, updated_at = NOW()
                WHERE id = $1
                RETURNING id, title_group_id, user_id, rating, review, created_at, updated_at
            "#,
            review_id,
            edited_review.rating,
            edited_review.review
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(Error::TitleGroupReviewNotFound)?;

        Self::update_title_group_user_rating(&mut transaction, updated_review.title_group_id)
            .await?;

        transaction.commit().await?;

        Ok(updated_review)
    }

    pub async fn delete_title_group_review(&self, review_id: i64) -> Result<()> {
        let mut transaction = self.pool.begin().await?;

        let deleted_review = sqlx::query!(
            "DELETE FROM title_group_reviews WHERE id = $1 RETURNING user_id, title_group_id",
            review_id
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(Error::TitleGroupReviewNotFound)?;

        sqlx::query!(
            "UPDATE users SET title_group_reviews = title_group_reviews - 1 WHERE id = $1",
            deleted_review.user_id
        )
        .execute(&mut *transaction)
        .await?;

        Self::update_title_group_user_rating(&mut transaction, deleted_review.title_group_id)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    /// Recomputes the cached average and amount of ratings of a title group from its reviews.
    pub async fn update_title_group_user_rating(
        connection: &mut PgConnection,
        title_group_id: i32,
    ) -> Result<()> {
        // Concurrent transactions reviewing the same title group wait here for each other, so
        // the recompute below runs with a fresh snapshot that includes their committed reviews
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
                    FROM title_group_reviews
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

    pub async fn find_user_title_group_review(
        &self,
        user_id: i32,
        title_group_id: i32,
    ) -> Result<Option<TitleGroupReview>> {
        Ok(sqlx::query_as!(
            TitleGroupReview,
            r#"
                SELECT id, title_group_id, user_id, rating, review, created_at, updated_at
                FROM title_group_reviews
                WHERE user_id = $1 AND title_group_id = $2
            "#,
            user_id,
            title_group_id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// The reviews of a title group matching the query, ties broken by the most recent first.
    pub async fn search_title_group_reviews(
        &self,
        form: &TitleGroupReviewSearchQuery,
    ) -> Result<PaginatedResults<TitleGroupReviewHierarchy>> {
        let limit = form.page_size as i64;
        let offset = (form.page as i64 - 1).max(0) * limit;

        let results = sqlx::query_as!(
            TitleGroupReviewHierarchy,
            r#"
                SELECT
                    r.id, r.rating, r.review, r.created_at, r.updated_at,
                    ROW(u.id, u.username, u.class_name, u.banned, u.avatar, u.warned, u.custom_title) AS "created_by!: UserLiteAvatar"
                FROM title_group_reviews r
                JOIN users u ON u.id = r.user_id
                WHERE r.title_group_id = $1
                  AND r.rating BETWEEN $2 AND $3
                  AND COALESCE(r.review, '') ILIKE '%' || $4 || '%'
                ORDER BY
                    CASE WHEN $7 = 'created_at' AND $8 = 'asc' THEN r.created_at END ASC,
                    CASE WHEN $7 = 'created_at' AND $8 = 'desc' THEN r.created_at END DESC,
                    CASE WHEN $7 = 'updated_at' AND $8 = 'asc' THEN r.updated_at END ASC,
                    CASE WHEN $7 = 'updated_at' AND $8 = 'desc' THEN r.updated_at END DESC,
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
                FROM title_group_reviews
                WHERE title_group_id = $1
                  AND rating BETWEEN $2 AND $3
                  AND COALESCE(review, '') ILIKE '%' || $4 || '%'
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

    /// Every title group review written by a user, most recent first.
    pub async fn search_user_title_group_reviews(
        &self,
        form: &UserTitleGroupReviewSearchQuery,
    ) -> Result<PaginatedResults<TitleGroupReviewWithTitleGroup>> {
        let limit = form.page_size as i64;
        let offset = (form.page as i64 - 1).max(0) * limit;

        let results = sqlx::query_as!(
            TitleGroupReviewWithTitleGroup,
            r#"
                SELECT
                    r.id, r.title_group_id, tg.name AS title_group_name, r.user_id, r.rating,
                    r.review, r.created_at, r.updated_at
                FROM title_group_reviews r
                JOIN title_groups tg ON tg.id = r.title_group_id
                WHERE r.user_id = $1
                ORDER BY r.created_at DESC
                LIMIT $2 OFFSET $3
            "#,
            form.user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total_items = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM title_group_reviews WHERE user_id = $1"#,
            form.user_id
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
