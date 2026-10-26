use crate::connection_pool::ConnectionPool;
use crate::models::artist::{AffiliatedArtistLite, ArtistLite};
use crate::models::collage::CollageLite;
use crate::models::common::PaginatedResults;
use crate::models::forum::{ForumSubCategoryLite, ForumThreadLite};
use crate::models::subscription::SearchSubscriptionsQuery;
use crate::models::title_group::TitleGroupHierarchyLite;
use arcadia_common::error::{Error, Result};
use sqlx::types::Json;
use std::collections::HashMap;

impl ConnectionPool {
    pub async fn find_subscription_forum_sub_category_threads(
        &self,
        current_user_id: i32,
        query: &SearchSubscriptionsQuery,
    ) -> Result<PaginatedResults<ForumSubCategoryLite>> {
        let page_size = query.page_size as i64;
        let offset = (query.page as i64 - 1).max(0) * page_size;

        let total_items = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*)::BIGINT
                FROM subscriptions_forum_sub_category_threads s
                WHERE s.user_id = $1
            "#,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let order_direction = query.order_by_direction.to_string();
        let results = sqlx::query_as!(
            ForumSubCategoryLite,
            r#"
                SELECT fsc.id, fsc.name
                FROM subscriptions_forum_sub_category_threads s
                JOIN forum_sub_categories fsc ON fsc.id = s.forum_sub_category_id
                WHERE s.user_id = $1
                ORDER BY
                    CASE WHEN $4 = 'asc' THEN s.created_at END ASC,
                    CASE WHEN $4 = 'desc' THEN s.created_at END DESC
                LIMIT $2 OFFSET $3
            "#,
            current_user_id,
            page_size,
            offset,
            order_direction
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }

    pub async fn create_subscription_forum_sub_category_threads(
        &self,
        forum_sub_category_id: i32,
        current_user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO subscriptions_forum_sub_category_threads (user_id, forum_sub_category_id)
                VALUES ($1, $2)
            "#,
            current_user_id,
            forum_sub_category_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotCreateSubscription)?;

        Ok(())
    }

    pub async fn delete_subscription_forum_sub_category_threads(
        &self,
        forum_sub_category_id: i32,
        current_user_id: i32,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                DELETE FROM subscriptions_forum_sub_category_threads
                WHERE forum_sub_category_id = $1 AND user_id = $2;
            "#,
            forum_sub_category_id,
            current_user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_subscription_forum_thread_posts(
        &self,
        current_user_id: i32,
        query: &SearchSubscriptionsQuery,
    ) -> Result<PaginatedResults<ForumThreadLite>> {
        let page_size = query.page_size as i64;
        let offset = (query.page as i64 - 1).max(0) * page_size;

        let total_items = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*)::BIGINT
                FROM subscriptions_forum_thread_posts s
                WHERE s.user_id = $1
            "#,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let order_direction = query.order_by_direction.to_string();
        let results = sqlx::query_as!(
            ForumThreadLite,
            r#"
                SELECT ft.id, ft.forum_sub_category_id, ft.name
                FROM subscriptions_forum_thread_posts s
                JOIN forum_threads ft ON ft.id = s.forum_thread_id
                WHERE s.user_id = $1
                ORDER BY
                    CASE WHEN $4 = 'asc' THEN s.created_at END ASC,
                    CASE WHEN $4 = 'desc' THEN s.created_at END DESC
                LIMIT $2 OFFSET $3
            "#,
            current_user_id,
            page_size,
            offset,
            order_direction
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }

    pub async fn find_subscription_title_group_torrents(
        &self,
        current_user_id: i32,
        query: &SearchSubscriptionsQuery,
    ) -> Result<PaginatedResults<TitleGroupHierarchyLite>> {
        let page_size = query.page_size as i64;
        let offset = (query.page as i64 - 1).max(0) * page_size;

        let total_items = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*)::BIGINT
                FROM subscriptions_title_group_torrents s
                WHERE s.user_id = $1
            "#,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let mut results = sqlx::query_as!(
            TitleGroupHierarchyLite,
            r#"
                SELECT DISTINCT ON (tgh.title_group_id)
                    tgh.title_group_id AS "id!",
                    tgh.title_group_name AS "name!",
                    tgh.title_group_covers AS "covers!",
                    tgh.title_group_category AS "category!: _",
                    tgh.title_group_content_type AS "content_type!: _",
                    tgh.title_group_tag_names AS "tags!",
                    tgh.title_group_original_release_date AS "original_release_date",
                    tgh.title_group_original_release_date_only_year_known AS "original_release_date_only_year_known!",
                    tgh.title_group_platform AS "platform!: _",
                    '[]'::jsonb AS "edition_groups!: _",
                    '[]'::jsonb AS "affiliated_artists!: _",
                    CASE
                        WHEN tgh.title_group_series_id IS NOT NULL THEN jsonb_build_object('id', tgh.title_group_series_id, 'name', tgh.title_group_series_name)
                        ELSE NULL
                    END AS "series: _"
                FROM subscriptions_title_group_torrents s
                JOIN title_group_hierarchy_lite tgh ON tgh.title_group_id = s.title_group_id
                WHERE s.user_id = $1
                ORDER BY tgh.title_group_id
                LIMIT $2 OFFSET $3
            "#,
            current_user_id,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        self.enrich_title_groups_with_affiliated_artists(&mut results)
            .await?;

        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }

    pub async fn find_subscription_title_group_comments(
        &self,
        current_user_id: i32,
        query: &SearchSubscriptionsQuery,
    ) -> Result<PaginatedResults<TitleGroupHierarchyLite>> {
        let page_size = query.page_size as i64;
        let offset = (query.page as i64 - 1).max(0) * page_size;

        let total_items = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*)::BIGINT
                FROM subscriptions_title_group_comments s
                WHERE s.user_id = $1
            "#,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let mut results = sqlx::query_as!(
            TitleGroupHierarchyLite,
            r#"
                SELECT DISTINCT ON (tgh.title_group_id)
                    tgh.title_group_id AS "id!",
                    tgh.title_group_name AS "name!",
                    tgh.title_group_covers AS "covers!",
                    tgh.title_group_category AS "category!: _",
                    tgh.title_group_content_type AS "content_type!: _",
                    tgh.title_group_tag_names AS "tags!",
                    tgh.title_group_original_release_date AS "original_release_date",
                    tgh.title_group_original_release_date_only_year_known AS "original_release_date_only_year_known!",
                    tgh.title_group_platform AS "platform!: _",
                    '[]'::jsonb AS "edition_groups!: _",
                    '[]'::jsonb AS "affiliated_artists!: _",
                    CASE
                        WHEN tgh.title_group_series_id IS NOT NULL THEN jsonb_build_object('id', tgh.title_group_series_id, 'name', tgh.title_group_series_name)
                        ELSE NULL
                    END AS "series: _"
                FROM subscriptions_title_group_comments s
                JOIN title_group_hierarchy_lite tgh ON tgh.title_group_id = s.title_group_id
                WHERE s.user_id = $1
                ORDER BY tgh.title_group_id
                LIMIT $2 OFFSET $3
            "#,
            current_user_id,
            page_size,
            offset,
        )
        .fetch_all(&self.pool)
        .await?;

        self.enrich_title_groups_with_affiliated_artists(&mut results)
            .await?;

        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }

    async fn enrich_title_groups_with_affiliated_artists(
        &self,
        title_groups: &mut [TitleGroupHierarchyLite],
    ) -> Result<()> {
        let title_group_ids: Vec<i32> = title_groups.iter().map(|tg| tg.id).collect();

        if title_group_ids.is_empty() {
            return Ok(());
        }

        let affiliated_artists = sqlx::query!(
            r#"
            WITH artist_counts AS (
                SELECT
                    title_group_id,
                    COUNT(*) as count
                FROM affiliated_artists
                WHERE title_group_id = ANY($1)
                GROUP BY title_group_id
            )
            SELECT
                aa.title_group_id,
                a.id as artist_id,
                a.name as artist_name
            FROM affiliated_artists aa
            JOIN artists a ON a.id = aa.artist_id
            JOIN artist_counts ac ON ac.title_group_id = aa.title_group_id
            WHERE ac.count <= 2

            UNION ALL

            SELECT DISTINCT ON (ac.title_group_id)
                ac.title_group_id,
                0::bigint as artist_id,
                ''::text as artist_name
            FROM artist_counts ac
            WHERE ac.count > 2
            ORDER BY title_group_id, artist_id
            "#,
            &title_group_ids
        )
        .fetch_all(&self.pool)
        .await?;

        let mut grouped_artists: HashMap<i32, Vec<AffiliatedArtistLite>> = HashMap::new();
        for row in affiliated_artists {
            if let (Some(title_group_id), Some(artist_id), Some(artist_name)) =
                (row.title_group_id, row.artist_id, row.artist_name)
            {
                grouped_artists
                    .entry(title_group_id)
                    .or_default()
                    .push(AffiliatedArtistLite {
                        artist_id,
                        name: artist_name,
                    });
            }
        }

        for title_group in title_groups.iter_mut() {
            title_group.affiliated_artists = Json(
                grouped_artists
                    .get(&title_group.id)
                    .cloned()
                    .unwrap_or_default(),
            );
        }

        Ok(())
    }

    pub async fn create_subscription_forum_thread_posts(
        &self,
        thread_id: i64,
        current_user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO subscriptions_forum_thread_posts (user_id, forum_thread_id)
                VALUES ($1, $2)
            "#,
            current_user_id,
            thread_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotCreateSubscription)?;

        Ok(())
    }

    pub async fn delete_subscription_forum_thread_posts(
        &self,
        thread_id: i64,
        current_user_id: i32,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                DELETE FROM subscriptions_forum_thread_posts
                WHERE forum_thread_id = $1 AND user_id = $2;
            "#,
            thread_id,
            current_user_id
        )
        .execute(&self.pool)
        .await?;

        // TODO: check result.rows_affected()
        Ok(())
    }

    pub async fn create_subscription_title_group_torrents(
        &self,
        title_group_id: i32,
        current_user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                   INSERT INTO subscriptions_title_group_torrents (user_id, title_group_id)
                   VALUES ($1, $2)
               "#,
            current_user_id,
            title_group_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotCreateSubscription)?;

        Ok(())
    }

    pub async fn delete_subscription_title_group_torrents(
        &self,
        title_group_id: i32,
        current_user_id: i32,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                DELETE FROM subscriptions_title_group_torrents
                WHERE title_group_id = $1 AND user_id = $2;
            "#,
            title_group_id,
            current_user_id
        )
        .execute(&self.pool)
        .await?;

        // TODO: check result.rows_affected()
        Ok(())
    }

    pub async fn create_subscription_title_group_comments(
        &self,
        title_group_id: i32,
        current_user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO subscriptions_title_group_comments (user_id, title_group_id)
                VALUES ($1, $2)
            "#,
            current_user_id,
            title_group_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotCreateSubscription)?;

        Ok(())
    }

    pub async fn delete_subscription_title_group_comments(
        &self,
        title_group_id: i32,
        current_user_id: i32,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                DELETE FROM subscriptions_title_group_comments
                WHERE title_group_id = $1 AND user_id = $2;
            "#,
            title_group_id,
            current_user_id
        )
        .execute(&self.pool)
        .await?;

        // TODO: check result.rows_affected()
        Ok(())
    }

    pub async fn find_subscription_artist_title_groups(
        &self,
        current_user_id: i32,
        query: &SearchSubscriptionsQuery,
    ) -> Result<PaginatedResults<ArtistLite>> {
        let page_size = query.page_size as i64;
        let offset = (query.page as i64 - 1).max(0) * page_size;

        let total_items = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*)::BIGINT
                FROM subscriptions_artist_title_groups s
                WHERE s.user_id = $1
            "#,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let order_direction = query.order_by_direction.to_string();
        let results = sqlx::query_as!(
            ArtistLite,
            r#"
                SELECT a.id, a.name, a.aliases, a.pictures
                FROM subscriptions_artist_title_groups s
                JOIN artists a ON a.id = s.artist_id
                WHERE s.user_id = $1
                ORDER BY
                    CASE WHEN $4 = 'asc' THEN s.created_at END ASC,
                    CASE WHEN $4 = 'desc' THEN s.created_at END DESC
                LIMIT $2 OFFSET $3
            "#,
            current_user_id,
            page_size,
            offset,
            order_direction
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }

    pub async fn create_subscription_artist_title_groups(
        &self,
        artist_id: i64,
        current_user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO subscriptions_artist_title_groups (user_id, artist_id)
                VALUES ($1, $2)
            "#,
            current_user_id,
            artist_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotCreateSubscription)?;

        Ok(())
    }

    pub async fn delete_subscription_artist_title_groups(
        &self,
        artist_id: i64,
        current_user_id: i32,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                DELETE FROM subscriptions_artist_title_groups
                WHERE artist_id = $1 AND user_id = $2;
            "#,
            artist_id,
            current_user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_subscription_collages(
        &self,
        current_user_id: i32,
        query: &SearchSubscriptionsQuery,
    ) -> Result<PaginatedResults<CollageLite>> {
        let page_size = query.page_size as i64;
        let offset = (query.page as i64 - 1).max(0) * page_size;
        let total_items = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*)::BIGINT
                FROM subscriptions_collages s
                WHERE s.user_id = $1
            "#,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);
        let order_direction = query.order_by_direction.to_string();
        let results = sqlx::query_as!(
            CollageLite,
            r#"
                SELECT c.id, c.name, c.cover
                FROM subscriptions_collages s
                JOIN collage c ON c.id = s.collage_id
                WHERE s.user_id = $1
                ORDER BY
                    CASE WHEN $4 = 'asc' THEN s.created_at END ASC,
                    CASE WHEN $4 = 'desc' THEN s.created_at END DESC
                LIMIT $2 OFFSET $3
            "#,
            current_user_id,
            page_size,
            offset,
            order_direction
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }
    pub async fn create_subscription_collages(
        &self,
        collage_id: i64,
        current_user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO subscriptions_collages (user_id, collage_id)
                VALUES ($1, $2)
            "#,
            current_user_id,
            collage_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotCreateSubscription)?;
        Ok(())
    }
    pub async fn delete_subscription_collages(
        &self,
        collage_id: i64,
        current_user_id: i32,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                DELETE FROM subscriptions_collages
                WHERE collage_id = $1 AND user_id = $2;
            "#,
            collage_id,
            current_user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn create_subscription_torrent_request_comments(
        &self,
        torrent_request_id: i64,
        current_user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO subscriptions_torrent_request_comments (user_id, torrent_request_id)
                VALUES ($1, $2)
            "#,
            current_user_id,
            torrent_request_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotCreateSubscription)?;

        Ok(())
    }

    pub async fn delete_subscription_torrent_request_comments(
        &self,
        torrent_request_id: i64,
        current_user_id: i32,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                DELETE FROM subscriptions_torrent_request_comments
                WHERE torrent_request_id = $1 AND user_id = $2;
            "#,
            torrent_request_id,
            current_user_id
        )
        .execute(&self.pool)
        .await?;

        // TODO: check result.rows_affected()
        Ok(())
    }
}
