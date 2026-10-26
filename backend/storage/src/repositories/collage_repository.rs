use crate::{
    connection_pool::ConnectionPool,
    models::{
        collage::{
            Collage, CollageCategory, CollageEnriched, CollageEntry, CollageLite,
            CollageSearchResult, EditedCollage, SearchCollagesLiteQuery, SearchCollagesQuery,
            UserCreatedCollage, UserCreatedCollageEntry,
        },
        common::PaginatedResults,
        notification::NotificationEvent,
        title_group_stats::CatalogKind,
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::{query_as_unchecked, query_scalar, Postgres, Transaction};
use tokio::sync::broadcast;

impl ConnectionPool {
    pub async fn create_collage(
        &self,
        collage: &UserCreatedCollage,
        user_id: i32,
    ) -> Result<Collage> {
        let created_collage = sqlx::query_as!(
            Collage,
            r#"
                WITH inserted_collage AS (
                    INSERT INTO collage (created_by_id, name, cover, description, tags, category)
                    VALUES ($1, $2, $3, $4, $5, $6::collage_category_enum)
                    RETURNING id, created_at, created_by_id, name, cover, description, tags, category
                ),
                updated_user AS (
                    UPDATE users u
                    SET collages_started = u.collages_started + 1
                    WHERE u.id = (SELECT created_by_id FROM inserted_collage)
                )
                SELECT
                    id, created_at, created_by_id, name, cover, description, tags,
                    category as "category: CollageCategory"
                FROM inserted_collage
            "#,
            user_id,
            collage.name,
            collage.cover,
            collage.description,
            &collage.tags,
            collage.category as _,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateCollage)?;

        Ok(created_collage)
    }

    pub async fn create_collage_entries(
        &self,
        collage_entries: &[UserCreatedCollageEntry],
        user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<Vec<CollageEntry>> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;
        let mut created_entries = Vec::with_capacity(collage_entries.len());
        let mut notified_user_ids: Vec<i32> = Vec::new();

        for entry in collage_entries {
            let created = sqlx::query_as!(
                CollageEntry,
                r#"
                    INSERT INTO collage_entry (
                        created_by_id,
                        title_group_id,
                        collage_id,
                        note
                    )
                    VALUES ($1, $2, $3, $4)
                    RETURNING id, created_at, created_by_id, title_group_id, collage_id, note
                "#,
                user_id,
                entry.title_group_id,
                entry.collage_id,
                entry.note
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| Error::CouldNotCreateCollageEntry(e.to_string()))?;

            let user_ids = Self::notify_users_collages(
                &mut tx,
                created.collage_id,
                created.title_group_id,
                user_id,
            )
            .await?;
            notified_user_ids.extend(user_ids);
            created_entries.push(created);
        }

        tx.commit().await?;
        if !notified_user_ids.is_empty() {
            let _ = notification_sender.send(NotificationEvent::Collage {
                user_ids: notified_user_ids,
            });
        }
        Ok(created_entries)
    }

    pub async fn find_collage(&self, collage_id: i64) -> Result<Collage> {
        let collage = sqlx::query_as!(
            Collage,
            r#"
            SELECT
                id,
                created_at,
                created_by_id,
                name,
                cover,
                description,
                tags,
                category as "category: CollageCategory"
            FROM collage
            WHERE id = $1
            "#,
            collage_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFetchCollage)?;

        Ok(collage)
    }

    pub async fn find_collage_enriched(
        &self,
        collage_id: i64,
        current_user_id: i32,
    ) -> Result<CollageEnriched> {
        let row = sqlx::query!(
            r#"
                SELECT
                    to_jsonb(c) AS "collage!: sqlx::types::Json<Collage>",
                    EXISTS (
                        SELECT 1
                        FROM subscriptions_collages s
                        WHERE s.collage_id = c.id
                          AND s.user_id = $2
                    ) AS "is_subscribed!"
                FROM collage c
                WHERE c.id = $1
            "#,
            collage_id,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFetchCollage)?;
        let title_group_stats = self
            .get_title_group_catalog_stats(CatalogKind::Collage, collage_id)
            .await?;
        Ok(CollageEnriched {
            collage: row.collage.0,
            is_subscribed: row.is_subscribed,
            title_group_stats,
        })
    }
    pub async fn search_collages(
        &self,
        form: &SearchCollagesQuery,
    ) -> Result<PaginatedResults<CollageSearchResult>> {
        let offset = (form.page - 1) * form.page_size;

        let total_items: i64 = query_scalar!(
            "
            SELECT COUNT(*)
            FROM collage c
            WHERE (c.name ILIKE '%' || $1 || '%')
            ",
            form.name,
        )
        .fetch_one(&self.pool)
        .await
        .unwrap()
        .unwrap();

        let results = query_as_unchecked!(
            CollageSearchResult,
            r#"
            SELECT
                c.id,
                c.created_at,
                c.created_by_id,
                ROW(u.id, u.username, u.warned, u.banned) AS created_by,
                c.name,
                c.cover,
                c.description,
                c.tags,
                c.category,
                COUNT(ce.id) AS entries_amount,
                MAX(ce.created_at) AS last_entry_at
            FROM
                collage c
            JOIN
                users u ON c.created_by_id = u.id
            LEFT JOIN
                collage_entry ce ON c.id = ce.collage_id
            WHERE
                (c.name ILIKE '%' || $1 || '%')
            GROUP BY
                c.id, u.id
            ORDER BY
                c.created_at DESC
            OFFSET $2
            LIMIT $3
            "#,
            form.name,
            offset as i64,
            form.page_size as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            total_items,
            page: form.page,
            page_size: form.page_size,
        })
    }

    pub async fn search_collages_lite(
        &self,
        form: &SearchCollagesLiteQuery,
    ) -> Result<Vec<CollageLite>> {
        let results = sqlx::query_as!(
            CollageLite,
            r#"
                SELECT
                    c.id,
                    c.name,
                    c.cover
                FROM
                    collage c
                WHERE
                    (c.name ILIKE '%' || $1 || '%')
                ORDER BY
                    CASE
                        -- Exact Match: Highest priority
                        WHEN c.name = $1 THEN 1
                        -- Starts With Match (Prefix): Second highest priority
                        WHEN c.name ILIKE $1 || '%' THEN 2
                        -- Anywhere Match: Lowest priority (or all remaining)
                        ELSE 3
                    END
                LIMIT $2
                "#,
            form.name,
            form.results_amount as i16
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    pub async fn update_collage(&self, form: &EditedCollage) -> Result<Collage> {
        let updated_collage = sqlx::query_as!(
            Collage,
            r#"
            UPDATE collage
            SET name = $1, cover = $2, description = $3, tags = $4, category = $5::collage_category_enum
            WHERE id = $6
            RETURNING id, created_at, created_by_id, name, cover, description, tags,
                      category as "category: CollageCategory"
            "#,
            form.name,
            form.cover,
            form.description,
            &form.tags,
            form.category as _,
            form.id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotUpdateCollage)?;

        Ok(updated_collage)
    }

    pub async fn delete_collage(&self, collage_id: i64) -> Result<()> {
        let entry_count: i64 = query_scalar!(
            r#"
            SELECT COUNT(*) FROM collage_entry WHERE collage_id = $1
            "#,
            collage_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteCollage)?
        .unwrap_or(0);

        if entry_count > 0 {
            return Err(Error::CollageHasEntries);
        }

        sqlx::query!(
            r#"
            DELETE FROM collage
            WHERE id = $1
            "#,
            collage_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteCollage)?;

        Ok(())
    }

    pub async fn delete_collage_entry(&self, collage_id: i64, title_group_id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM collage_entry
            WHERE collage_id = $1 AND title_group_id = $2
            "#,
            collage_id,
            title_group_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteCollageEntry)?;

        Ok(())
    }

    pub async fn find_collage_entry(
        &self,
        collage_id: i64,
        title_group_id: i32,
    ) -> Result<CollageEntry> {
        let entry = sqlx::query_as!(
            CollageEntry,
            r#"
            SELECT id, created_at, created_by_id, title_group_id, collage_id, note
            FROM collage_entry
            WHERE collage_id = $1 AND title_group_id = $2
            "#,
            collage_id,
            title_group_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFetchCollage)?;

        Ok(entry)
    }
}
