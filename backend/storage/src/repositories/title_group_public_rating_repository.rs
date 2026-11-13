use crate::{
    connection_pool::ConnectionPool,
    models::title_group_public_rating::{CollectedPublicRating, PublicRatingToFetch},
};
use arcadia_common::error::Result;

impl ConnectionPool {
    /// Creates the rows of the given (title group, external link, provider) triples, the existing
    /// ones are left untouched. Returns how many were created.
    pub async fn create_public_ratings_to_fetch(
        &self,
        title_group_ids: &[i32],
        external_links: &[String],
        providers: &[String],
    ) -> Result<u64> {
        let created = sqlx::query!(
            r#"
            INSERT INTO title_group_public_ratings (title_group_id, external_link, provider)
            SELECT * FROM unnest($1::INT[], $2::TEXT[], $3::TEXT[])
            ON CONFLICT DO NOTHING
            "#,
            title_group_ids,
            external_links,
            providers
        )
        .execute(&self.pool)
        .await?;

        Ok(created.rows_affected())
    }

    /// Removes the ratings of the links the title group no longer has.
    pub async fn delete_public_ratings_of_removed_links(
        &self,
        title_group_id: i32,
        external_links: &[String],
    ) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM title_group_public_ratings
            WHERE title_group_id = $1 AND NOT (external_link = ANY($2))
            "#,
            title_group_id,
            external_links
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Claims up to `limit` ratings of the given providers that were never asked for, or not for
    /// `retry_seconds`, the least recently asked first, by marking them as tried now. Ratings
    /// claimed by a concurrent fetch are skipped. Restricted to one title group when
    /// `title_group_id` is set.
    pub async fn claim_public_ratings_to_fetch(
        &self,
        title_group_id: Option<i32>,
        providers: &[String],
        retry_seconds: f64,
        limit: i64,
    ) -> Result<Vec<PublicRatingToFetch>> {
        let ratings = sqlx::query_as!(
            PublicRatingToFetch,
            r#"
            UPDATE title_group_public_ratings rating
            SET last_tried_fetching_at = NOW()
            FROM (
                SELECT title_group_id, external_link, provider
                FROM title_group_public_ratings
                WHERE ($1::INT IS NULL OR title_group_id = $1)
                  AND provider = ANY($2)
                  AND (
                      last_tried_fetching_at IS NULL
                      OR last_tried_fetching_at < NOW() - make_interval(secs => $3)
                  )
                ORDER BY last_tried_fetching_at NULLS FIRST
                LIMIT $4
                FOR UPDATE SKIP LOCKED
            ) due
            WHERE rating.title_group_id = due.title_group_id
              AND rating.external_link = due.external_link
              AND rating.provider = due.provider
            RETURNING
                rating.title_group_id, rating.external_link, rating.provider, rating.collected_at,
                (
                    SELECT original_release_date FROM title_groups
                    WHERE id = rating.title_group_id
                ) AS "title_group_original_release_date?"
            "#,
            title_group_id,
            providers,
            retry_seconds,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(ratings)
    }

    pub async fn save_public_rating(
        &self,
        rating_to_fetch: &PublicRatingToFetch,
        collected_rating: &CollectedPublicRating,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE title_group_public_ratings
            SET rating = $4, votes = $5, collected_at = NOW()
            WHERE title_group_id = $1 AND external_link = $2 AND provider = $3
            "#,
            rating_to_fetch.title_group_id,
            rating_to_fetch.external_link,
            rating_to_fetch.provider,
            collected_rating.rating,
            collected_rating.votes
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_public_rating(&self, rating_to_fetch: &PublicRatingToFetch) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM title_group_public_ratings
            WHERE title_group_id = $1 AND external_link = $2 AND provider = $3
            "#,
            rating_to_fetch.title_group_id,
            rating_to_fetch.external_link,
            rating_to_fetch.provider
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// The id and external links of every title group having some.
    pub async fn find_title_groups_external_links(&self) -> Result<Vec<(i32, Vec<String>)>> {
        let title_groups = sqlx::query!(
            r#"
            SELECT id, external_links
            FROM title_groups
            WHERE cardinality(external_links) > 0
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(title_groups
            .into_iter()
            .map(|title_group| (title_group.id, title_group.external_links))
            .collect())
    }
}
