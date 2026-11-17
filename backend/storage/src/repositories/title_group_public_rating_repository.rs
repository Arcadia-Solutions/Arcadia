use crate::{
    connection_pool::ConnectionPool,
    models::title_group_public_rating::{CollectedPublicRating, PublicRatingToFetch},
};
use arcadia_common::error::Result;
use chrono::{DateTime, Utc};

impl ConnectionPool {
    /// Creates the rows of the given (external link, provider) pairs, the existing ones are left
    /// untouched. Returns how many were created.
    pub async fn create_public_ratings_to_fetch(
        &self,
        external_links: &[String],
        providers: &[String],
    ) -> Result<u64> {
        let created = sqlx::query!(
            r#"
            INSERT INTO public_ratings (external_link, provider)
            SELECT * FROM unnest($1::TEXT[], $2::TEXT[])
            ON CONFLICT DO NOTHING
            "#,
            external_links,
            providers
        )
        .execute(&self.pool)
        .await?;

        Ok(created.rows_affected())
    }

    /// Claims up to `limit` ratings of the given providers, of links listed by a title group, that
    /// were never asked for, or not for `retry_seconds` before `run_started_at`, the least recently
    /// asked first, by marking them as tried now. Ratings claimed by a concurrent fetch are
    /// skipped. Restricted to the given links when `external_links` is set.
    pub async fn claim_public_ratings_to_fetch(
        &self,
        external_links: Option<&[String]>,
        providers: &[String],
        retry_seconds: f64,
        run_started_at: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<PublicRatingToFetch>> {
        let ratings = sqlx::query_as!(
            PublicRatingToFetch,
            r#"
            UPDATE public_ratings rating
            SET last_tried_fetching_at = NOW()
            FROM (
                SELECT external_link
                FROM public_ratings
                WHERE ($1::TEXT[] IS NULL OR external_link = ANY($1))
                  AND provider = ANY($2)
                  AND (
                      last_tried_fetching_at IS NULL
                      OR last_tried_fetching_at < $5::TIMESTAMPTZ - make_interval(secs => $3)
                  )
                  AND EXISTS (
                      SELECT 1 FROM title_groups
                      WHERE external_links @> ARRAY[public_ratings.external_link]
                  )
                ORDER BY last_tried_fetching_at NULLS FIRST
                LIMIT $4
                FOR UPDATE SKIP LOCKED
            ) due
            WHERE rating.external_link = due.external_link
            RETURNING
                rating.external_link, rating.provider, rating.collected_at,
                (
                    SELECT MAX(original_release_date) FROM title_groups
                    WHERE external_links @> ARRAY[rating.external_link]
                ) AS "original_release_date?"
            "#,
            external_links,
            providers,
            retry_seconds,
            limit,
            run_started_at
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(ratings)
    }

    /// Stores the rating the provider answered with, creating the row if the link has none yet.
    pub async fn save_public_rating(
        &self,
        external_link: &str,
        provider: &str,
        collected_rating: &CollectedPublicRating,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO public_ratings (
                external_link, provider, rating, votes, collected_at, last_tried_fetching_at
            )
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (external_link) DO UPDATE SET
                provider = EXCLUDED.provider,
                rating = EXCLUDED.rating,
                votes = EXCLUDED.votes,
                collected_at = EXCLUDED.collected_at,
                last_tried_fetching_at = EXCLUDED.last_tried_fetching_at
            "#,
            external_link,
            provider,
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
            DELETE FROM public_ratings
            WHERE external_link = $1 AND provider = $2
            "#,
            rating_to_fetch.external_link,
            rating_to_fetch.provider
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Deletes the ratings of the links no title group lists, last asked for (or collected while
    /// scraping) more than `grace_seconds` ago, so a title being uploaded keeps the rating scraped
    /// with it.
    pub async fn delete_unlisted_public_ratings(&self, grace_seconds: f64) -> Result<u64> {
        let deleted = sqlx::query!(
            r#"
            DELETE FROM public_ratings
            WHERE (
                last_tried_fetching_at IS NULL
                OR last_tried_fetching_at < NOW() - make_interval(secs => $1)
            )
              AND NOT EXISTS (
                  SELECT 1 FROM title_groups
                  WHERE external_links @> ARRAY[public_ratings.external_link]
              )
            "#,
            grace_seconds
        )
        .execute(&self.pool)
        .await?;

        Ok(deleted.rows_affected())
    }

    /// Every link listed by a title group.
    pub async fn find_all_title_groups_external_links(&self) -> Result<Vec<String>> {
        let links = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT unnest(external_links) AS "external_link!"
            FROM title_groups
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(links)
    }
}
