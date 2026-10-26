use crate::{
    connection_pool::ConnectionPool,
    models::{
        artist::AffiliatedArtistLite,
        common::PaginatedResults,
        series::SeriesLite,
        title_group::{ContentType, Platform, TitleGroupCategory, TitleGroupHierarchyLite},
        title_group_tag::{
            DeleteTitleGroupTagRequest, EditedTitleGroupTag, MergeTitleGroupTagsRequest,
            SearchTitleGroupAppliedTagsQuery, SearchTitleGroupTagsQuery, TitleGroupAppliedTag,
            TitleGroupTag, TitleGroupTagEnriched, TitleGroupTagLite, UserCreatedTitleGroupTag,
        },
        user::UserLite,
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::types::Json;
use std::collections::HashMap;

impl ConnectionPool {
    /// Tags are stored lowercase, with their words separated by dots. Dashes
    /// separate words just like spaces do, so that the `close-up` of a scraper
    /// and the `close up` of another end up on the same tag.
    fn sanitize_tag_name(name: &str) -> String {
        name.trim()
            .to_lowercase()
            .split(|character: char| character.is_whitespace() || character == '-')
            .filter(|word| !word.is_empty())
            .collect::<Vec<&str>>()
            .join(".")
    }

    pub async fn create_title_group_tag(
        &self,
        tag: &UserCreatedTitleGroupTag,
        user_id: i32,
    ) -> Result<TitleGroupTag> {
        let sanitized_name = Self::sanitize_tag_name(&tag.name);

        // Check if a soft-deleted tag with this name exists
        let deleted_tag = sqlx::query_scalar!(
            r#"
            SELECT deletion_reason
            FROM title_group_tags
            WHERE name = $1 AND deleted_at IS NOT NULL
            "#,
            sanitized_name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::CouldNotCreateTitleGroupTag)?;

        if let Some(deletion_reason) = deleted_tag {
            let reason = deletion_reason.unwrap_or_default();
            return Err(Error::TitleGroupTagDeleted(sanitized_name.clone(), reason));
        }

        let mut created_tag = sqlx::query_as!(
            TitleGroupTag,
            r#"
            INSERT INTO title_group_tags (name, created_by_id)
            VALUES ($1, $2)
            ON CONFLICT (name) DO NOTHING
            RETURNING
                id,
                name,
                synonyms as "synonyms!: Vec<String>",
                created_at,
                created_by_id
            "#,
            sanitized_name,
            user_id
        )
        .fetch_one(&self.pool)
        .await;

        // the tag already exists
        if created_tag.is_err() {
            created_tag = sqlx::query_as!(
                TitleGroupTag,
                r#"
                SELECT
                    id,
                    name,
                    synonyms as "synonyms!: Vec<String>",
                    created_at,
                    created_by_id
                FROM title_group_tags
                WHERE name = $1
                "#,
                sanitized_name
            )
            .fetch_one(&self.pool)
            .await;
        }

        created_tag.map_err(Error::CouldNotCreateTitleGroupTag)
    }

    /// Puts the given names in the form the tags are stored in: sanitized,
    /// replaced by the tag they are a synonym of, without the deleted ones and
    /// without the duplicates the replacement can create.
    pub async fn resolve_tag_names(&self, tag_names: &[String]) -> Result<Vec<String>> {
        let sanitized_names = tag_names
            .iter()
            .map(|tag_name| Self::sanitize_tag_name(tag_name))
            .filter(|sanitized_name| !sanitized_name.is_empty())
            .collect::<Vec<String>>();

        if sanitized_names.is_empty() {
            return Ok(Vec::new());
        }

        let matching_tags = sqlx::query!(
            r#"
            SELECT
                name,
                synonyms as "synonyms!: Vec<String>",
                (deleted_at IS NOT NULL) as "is_deleted!"
            FROM title_group_tags
            WHERE name = ANY($1) OR synonyms && $1::varchar[]
            "#,
            &sanitized_names
        )
        .fetch_all(&self.pool)
        .await?;

        // the name of the tag each name belongs to, `None` when that tag is deleted. A name can be
        // the one of a deleted tag and the synonym of a live one, in which case the live tag wins:
        // the live tags are inserted last.
        let mut tags_by_name: HashMap<&str, Option<&str>> = HashMap::new();
        for tag in matching_tags.iter().filter(|tag| tag.is_deleted) {
            for name in std::iter::once(&tag.name).chain(tag.synonyms.iter()) {
                tags_by_name.insert(name.as_str(), None);
            }
        }
        for tag in matching_tags.iter().filter(|tag| !tag.is_deleted) {
            for name in std::iter::once(&tag.name).chain(tag.synonyms.iter()) {
                tags_by_name.insert(name.as_str(), Some(tag.name.as_str()));
            }
        }

        let mut resolved_names: Vec<String> = Vec::new();
        for sanitized_name in &sanitized_names {
            let resolved_name = match tags_by_name.get(sanitized_name.as_str()) {
                Some(None) => continue,
                Some(Some(tag_name)) => (*tag_name).to_string(),
                None => sanitized_name.clone(),
            };

            if !resolved_names.contains(&resolved_name) {
                resolved_names.push(resolved_name);
            }
        }

        Ok(resolved_names)
    }

    async fn find_tag_id_by_name(&self, tag_name: &str) -> Result<Option<i32>> {
        let sanitized_name = Self::sanitize_tag_name(tag_name);

        let tag_id = sqlx::query_scalar!(
            r#"
            SELECT id FROM title_group_tags WHERE name = $1 AND deleted_at IS NULL
            "#,
            sanitized_name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(tag_id)
    }

    pub async fn find_title_group_tag(&self, tag_id: i32) -> Result<TitleGroupTag> {
        sqlx::query_as!(
            TitleGroupTag,
            r#"
            SELECT
                id,
                name,
                synonyms as "synonyms!: Vec<String>",
                created_at,
                created_by_id
            FROM title_group_tags
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            tag_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::TitleGroupTagNotFound)
    }

    pub async fn update_title_group_tag(
        &self,
        edited_tag: &EditedTitleGroupTag,
    ) -> Result<TitleGroupTag> {
        let sanitized_name = Self::sanitize_tag_name(&edited_tag.name);

        let updated_tag = sqlx::query_as!(
            TitleGroupTag,
            r#"
            UPDATE title_group_tags
            SET name = $1, synonyms = $2
            WHERE id = $3 AND deleted_at IS NULL
            RETURNING
                id,
                name,
                synonyms as "synonyms!: Vec<String>",
                created_at,
                created_by_id
            "#,
            sanitized_name,
            &edited_tag.synonyms as _,
            edited_tag.id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::CouldNotUpdateTitleGroupTag)?
        .ok_or(Error::TitleGroupTagNotFound)?;

        Ok(updated_tag)
    }

    pub async fn delete_title_group_tag(
        &self,
        request: &DeleteTitleGroupTagRequest,
        user_id: i32,
    ) -> Result<()> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE title_group_tags
            SET deleted_at = NOW(), deleted_by_id = $2, deletion_reason = $3
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            request.id,
            user_id,
            request.deletion_reason
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteTitleGroupTag)?;

        if rows_affected.rows_affected() == 0 {
            return Err(Error::TitleGroupTagNotFound);
        }

        sqlx::query!(
            r#"
            DELETE FROM title_group_applied_tags
            WHERE tag_id = $1
            "#,
            request.id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteTitleGroupTag)?;

        Ok(())
    }

    pub async fn restore_title_group_tag(&self, tag_id: i32) -> Result<TitleGroupTag> {
        // the tag can only come back if no live tag took its name or one of its
        // synonyms in the meantime, a merge being the usual way for that to happen
        let conflicting_tag_name = sqlx::query_scalar!(
            r#"
            SELECT live.name
            FROM title_group_tags deleted
            JOIN title_group_tags live
                ON live.id <> deleted.id
                AND live.deleted_at IS NULL
                AND (deleted.name = ANY(live.synonyms) OR live.synonyms && deleted.synonyms)
            WHERE deleted.id = $1
            LIMIT 1
            "#,
            tag_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(conflicting_tag_name) = conflicting_tag_name {
            return Err(Error::TitleGroupTagConflict(conflicting_tag_name));
        }

        sqlx::query_as!(
            TitleGroupTag,
            r#"
            UPDATE title_group_tags
            SET deleted_at = NULL, deleted_by_id = NULL, deletion_reason = NULL
            WHERE id = $1 AND deleted_at IS NOT NULL
            RETURNING
                id,
                name,
                synonyms as "synonyms!: Vec<String>",
                created_at,
                created_by_id
            "#,
            tag_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::CouldNotUpdateTitleGroupTag)?
        .ok_or(Error::TitleGroupTagNotFound)
    }

    /// Moves the title groups of the source tag onto the target one, hands the
    /// name and the synonyms of the source tag to the target one, and soft
    /// deletes the source tag.
    pub async fn merge_title_group_tags(
        &self,
        request: &MergeTitleGroupTagsRequest,
        user_id: i32,
    ) -> Result<TitleGroupTag> {
        if request.source_tag_id == request.target_tag_id {
            return Err(Error::TitleGroupTagCannotBeMergedIntoItself);
        }

        let mut transaction = self.pool.begin().await?;

        let source_tag = sqlx::query!(
            r#"
            SELECT name, synonyms as "synonyms!: Vec<String>"
            FROM title_group_tags
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            request.source_tag_id
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(Error::TitleGroupTagNotFound)?;

        let target_tag_name = sqlx::query_scalar!(
            r#"
            SELECT name FROM title_group_tags WHERE id = $1 AND deleted_at IS NULL
            "#,
            request.target_tag_id
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(Error::TitleGroupTagNotFound)?;

        sqlx::query!(
            r#"
            INSERT INTO title_group_applied_tags (title_group_id, tag_id, created_by_id)
            SELECT title_group_id, $2, created_by_id
            FROM title_group_applied_tags
            WHERE tag_id = $1
            ON CONFLICT DO NOTHING
            "#,
            request.source_tag_id,
            request.target_tag_id
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"
            DELETE FROM title_group_applied_tags WHERE tag_id = $1
            "#,
            request.source_tag_id
        )
        .execute(&mut *transaction)
        .await?;

        // the source tag is soft deleted before its names are given away, so
        // that the trigger enforcing unique synonyms does not see them twice
        sqlx::query!(
            r#"
            UPDATE title_group_tags
            SET deleted_at = NOW(), deleted_by_id = $2, deletion_reason = $3
            WHERE id = $1
            "#,
            request.source_tag_id,
            user_id,
            format!("merged into {target_tag_name}")
        )
        .execute(&mut *transaction)
        .await?;

        let mut given_names = source_tag.synonyms;
        given_names.push(source_tag.name);

        let target_tag = sqlx::query_as!(
            TitleGroupTag,
            r#"
            UPDATE title_group_tags
            SET synonyms = ARRAY(SELECT DISTINCT unnest(synonyms || $2::varchar[]))
            WHERE id = $1
            RETURNING
                id,
                name,
                synonyms as "synonyms!: Vec<String>",
                created_at,
                created_by_id
            "#,
            request.target_tag_id,
            &given_names as _
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(Error::CouldNotUpdateTitleGroupTag)?;

        transaction.commit().await?;

        Ok(target_tag)
    }

    pub async fn apply_tag_to_title_group(
        &self,
        title_group_id: i32,
        tag_id: i32,
        user_id: i32,
    ) -> Result<()> {
        let tag_is_applicable = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM title_group_tags WHERE id = $1 AND deleted_at IS NULL
            ) AS "tag_is_applicable!"
            "#,
            tag_id
        )
        .fetch_one(&self.pool)
        .await?;

        if !tag_is_applicable {
            return Err(Error::TitleGroupTagNotFound);
        }

        sqlx::query!(
            r#"
            INSERT INTO title_group_applied_tags (title_group_id, tag_id, created_by_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
            title_group_id,
            tag_id,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_tag_from_title_group(
        &self,
        title_group_id: i32,
        tag_name: &str,
    ) -> Result<()> {
        let tag_id = self.find_tag_id_by_name(tag_name).await?;

        let tag_id = tag_id.ok_or(Error::TitleGroupTagNotFound)?;

        sqlx::query!(
            r#"
            DELETE FROM title_group_applied_tags
            WHERE title_group_id = $1 AND tag_id = $2
            "#,
            title_group_id,
            tag_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn search_title_group_tags_lite(
        &self,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> Result<PaginatedResults<TitleGroupTagLite>> {
        let offset = ((page - 1) * page_size) as i64;
        let limit = page_size as i64;

        let total_items = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM title_group_tags
            WHERE
                deleted_at IS NULL
                AND (
                    name ILIKE '%' || $1 || '%'
                    OR EXISTS (
                        SELECT 1
                        FROM unnest(synonyms) AS synonym
                        WHERE synonym ILIKE '%' || $1 || '%'
                    )
                )
            "#,
            query
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let results = sqlx::query_as!(
            TitleGroupTagLite,
            r#"
            SELECT
                name,
                synonyms as "synonyms!: Vec<String>",
                id
            FROM title_group_tags
            WHERE
                deleted_at IS NULL
                AND (
                    name ILIKE '%' || $1 || '%'
                    OR EXISTS (
                        SELECT 1
                        FROM unnest(synonyms) AS synonym
                        WHERE synonym ILIKE '%' || $1 || '%'
                    )
                )
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#,
            query,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            page,
            page_size,
            total_items,
        })
    }

    pub async fn search_title_group_tags(
        &self,
        query: &SearchTitleGroupTagsQuery,
    ) -> Result<PaginatedResults<TitleGroupTagEnriched>> {
        let offset = ((query.page - 1) * query.page_size) as i64;
        let limit = query.page_size as i64;

        let total_items = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM title_group_tags t
            WHERE
                ($2 OR t.deleted_at IS NULL)
                AND (
                    t.name ILIKE '%' || $1 || '%'
                    OR EXISTS (
                        SELECT 1
                        FROM unnest(t.synonyms) AS synonym
                        WHERE synonym ILIKE '%' || $1 || '%'
                    )
                )
            "#,
            query.name,
            query.show_deleted
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let results = sqlx::query_as!(
            TitleGroupTagEnriched,
            r#"
            SELECT
                t.id,
                t.name,
                t.synonyms AS "synonyms!: Vec<String>",
                t.created_at,
                ROW(u.id, u.username, u.warned, u.banned) AS "created_by!: UserLite",
                COALESCE((
                    SELECT COUNT(*)::INT
                    FROM title_group_applied_tags at
                    WHERE at.tag_id = t.id
                ), 0) AS "uses!",
                t.deleted_at,
                CASE WHEN t.deleted_by_id IS NOT NULL
                    THEN ROW(deleter.id, deleter.username, deleter.warned, deleter.banned)
                END AS "deleted_by?: UserLite",
                t.deletion_reason
            FROM title_group_tags t
            JOIN users u ON t.created_by_id = u.id
            LEFT JOIN users deleter ON t.deleted_by_id = deleter.id
            WHERE
                ($6 OR t.deleted_at IS NULL)
                AND (
                    t.name ILIKE '%' || $1 || '%'
                    OR EXISTS (
                        SELECT 1
                        FROM unnest(t.synonyms) AS synonym
                        WHERE synonym ILIKE '%' || $1 || '%'
                    )
                )
            ORDER BY
                CASE WHEN $2 = 'name' AND $3 = 'asc' THEN t.name END ASC,
                CASE WHEN $2 = 'name' AND $3 = 'desc' THEN t.name END DESC,
                CASE WHEN $2 = 'created_at' AND $3 = 'asc' THEN t.created_at END ASC,
                CASE WHEN $2 = 'created_at' AND $3 = 'desc' THEN t.created_at END DESC,
                CASE WHEN $2 = 'uses' AND $3 = 'asc' THEN (
                    SELECT COUNT(*)::INT
                    FROM title_group_applied_tags at
                    WHERE at.tag_id = t.id
                ) END ASC,
                CASE WHEN $2 = 'uses' AND $3 = 'desc' THEN (
                    SELECT COUNT(*)::INT
                    FROM title_group_applied_tags at
                    WHERE at.tag_id = t.id
                ) END DESC,
                CASE WHEN $2 = 'created_by' AND $3 = 'asc' THEN u.username END ASC,
                CASE WHEN $2 = 'created_by' AND $3 = 'desc' THEN u.username END DESC,
                CASE WHEN $2 = 'deleted_by' AND $3 = 'asc' THEN deleter.username END ASC,
                CASE WHEN $2 = 'deleted_by' AND $3 = 'desc' THEN deleter.username END DESC,
                CASE WHEN $2 = 'deleted_at' AND $3 = 'asc' THEN t.deleted_at END ASC,
                CASE WHEN $2 = 'deleted_at' AND $3 = 'desc' THEN t.deleted_at END DESC,
                t.name
            LIMIT $4 OFFSET $5
            "#,
            query.name,
            query.order_by_column.to_string(),
            query.order_by_direction.to_string(),
            limit,
            offset,
            query.show_deleted
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

    pub async fn search_title_group_applied_tags(
        &self,
        query: &SearchTitleGroupAppliedTagsQuery,
    ) -> Result<PaginatedResults<TitleGroupAppliedTag>> {
        let offset = ((query.page - 1) * query.page_size) as i64;
        let limit = query.page_size as i64;

        let total_items = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM title_group_applied_tags applied_tag
            JOIN title_group_tags tag ON tag.id = applied_tag.tag_id
            JOIN users u ON u.id = applied_tag.created_by_id
            JOIN title_groups tg ON tg.id = applied_tag.title_group_id
            WHERE ($1::TEXT IS NULL OR tag.name = $1)
              AND ($2::TEXT IS NULL OR u.username ILIKE '%' || $2 || '%')
              AND ($3::TEXT IS NULL OR tg.name ILIKE '%' || $3 || '%')
              AND ($4::content_type_enum IS NULL OR tg.content_type = $4)
            "#,
            query.tag_name,
            query.username,
            query.title_group_name,
            &query.content_type as &Option<ContentType>
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT
                applied_tag.tag_id,
                tag.name AS tag_name,
                applied_tag.created_at,
                ROW(u.id, u.username, u.warned, u.banned) AS "created_by!: UserLite",
                user_applied_tags.amount AS "user_applied_tags_amount!",
                tg.id AS title_group_id,
                tg.name AS title_group_name,
                tg.covers,
                tg.category AS "category: TitleGroupCategory",
                tg.content_type AS "content_type: ContentType",
                tg.original_release_date,
                tg.original_release_date_only_year_known,
                tg.platform AS "platform: Platform",
                title_group_tags.names AS "title_group_tag_names!",
                COALESCE((
                    SELECT jsonb_agg(jsonb_build_object('artist_id', a.id, 'name', a.name))
                    FROM affiliated_artists aa
                    JOIN artists a ON a.id = aa.artist_id
                    WHERE aa.title_group_id = tg.id
                ), '[]'::jsonb) AS "affiliated_artists!: Json<Vec<AffiliatedArtistLite>>",
                CASE WHEN s.id IS NOT NULL
                    THEN jsonb_build_object('id', s.id, 'name', s.name)
                END AS "series: Json<SeriesLite>"
            FROM title_group_applied_tags applied_tag
            JOIN title_group_tags tag ON tag.id = applied_tag.tag_id
            JOIN users u ON u.id = applied_tag.created_by_id
            JOIN title_groups tg ON tg.id = applied_tag.title_group_id
            LEFT JOIN series s ON s.id = tg.series_id
            CROSS JOIN LATERAL (
                SELECT ARRAY_AGG(other_tag.name ORDER BY other_tag.name) AS names
                FROM title_group_applied_tags other_applied_tag
                JOIN title_group_tags other_tag ON other_tag.id = other_applied_tag.tag_id
                WHERE other_applied_tag.title_group_id = tg.id
            ) title_group_tags
            CROSS JOIN LATERAL (
                SELECT COUNT(*) AS amount
                FROM title_group_applied_tags user_applied_tag
                WHERE user_applied_tag.created_by_id = applied_tag.created_by_id
                  AND ($9::TIMESTAMPTZ IS NULL OR user_applied_tag.created_at >= $9)
                  AND ($10::TIMESTAMPTZ IS NULL OR user_applied_tag.created_at <= $10)
            ) user_applied_tags
            WHERE ($1::TEXT IS NULL OR tag.name = $1)
              AND ($2::TEXT IS NULL OR u.username ILIKE '%' || $2 || '%')
              AND ($3::TEXT IS NULL OR tg.name ILIKE '%' || $3 || '%')
              AND ($8::content_type_enum IS NULL OR tg.content_type = $8)
            ORDER BY
                CASE WHEN $4 = 'tag_name' AND $5 = 'asc' THEN tag.name END ASC,
                CASE WHEN $4 = 'tag_name' AND $5 = 'desc' THEN tag.name END DESC,
                CASE WHEN $4 = 'username' AND $5 = 'asc' THEN u.username END ASC,
                CASE WHEN $4 = 'username' AND $5 = 'desc' THEN u.username END DESC,
                CASE WHEN $4 = 'title_group_name' AND $5 = 'asc' THEN tg.name END ASC,
                CASE WHEN $4 = 'title_group_name' AND $5 = 'desc' THEN tg.name END DESC,
                CASE WHEN $4 = 'created_at' AND $5 = 'asc' THEN applied_tag.created_at END ASC,
                CASE WHEN $4 = 'created_at' AND $5 = 'desc' THEN applied_tag.created_at END DESC,
                CASE WHEN $4 = 'title_group_tags_amount' AND $5 = 'asc' THEN cardinality(title_group_tags.names) END ASC,
                CASE WHEN $4 = 'title_group_tags_amount' AND $5 = 'desc' THEN cardinality(title_group_tags.names) END DESC,
                CASE WHEN $4 = 'content_type' AND $5 = 'asc' THEN tg.content_type END ASC,
                CASE WHEN $4 = 'content_type' AND $5 = 'desc' THEN tg.content_type END DESC,
                CASE WHEN $4 = 'user_applied_tags_amount' AND $5 = 'asc' THEN user_applied_tags.amount END ASC,
                CASE WHEN $4 = 'user_applied_tags_amount' AND $5 = 'desc' THEN user_applied_tags.amount END DESC,
                applied_tag.created_at DESC
            LIMIT $6 OFFSET $7
            "#,
            query.tag_name,
            query.username,
            query.title_group_name,
            query.order_by_column.to_string(),
            query.order_by_direction.to_string(),
            limit,
            offset,
            &query.content_type as &Option<ContentType>,
            query.user_applied_tags_after,
            query.user_applied_tags_before
        )
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| TitleGroupAppliedTag {
                tag_id: row.tag_id,
                tag_name: row.tag_name,
                created_at: row.created_at,
                created_by: row.created_by,
                user_applied_tags_amount: row.user_applied_tags_amount,
                title_group: TitleGroupHierarchyLite {
                    id: row.title_group_id,
                    name: row.title_group_name,
                    covers: row.covers,
                    category: row.category,
                    content_type: row.content_type,
                    tags: row.title_group_tag_names,
                    original_release_date: row.original_release_date,
                    original_release_date_only_year_known: row
                        .original_release_date_only_year_known,
                    platform: row.platform,
                    edition_groups: Json(vec![]),
                    affiliated_artists: row.affiliated_artists,
                    series: row.series,
                },
            })
            .collect();

        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }
}
