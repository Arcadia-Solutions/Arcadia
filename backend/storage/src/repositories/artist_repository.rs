use crate::models::artist::EditedArtist;
use crate::{
    connection_pool::ConnectionPool,
    models::{
        artist::{
            AffiliatedArtist, AffiliatedArtistHierarchy, AffiliatedArtistLite, Artist,
            ArtistEnriched, ArtistLite, ArtistSearchResult, SearchArtistsQuery,
            UserCreatedAffiliatedArtist, UserCreatedArtist,
        },
        common::PaginatedResults,
        forum::RelatedForumThread,
        notification::NotificationEvent,
        title_group_stats::CatalogKind,
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;
use tokio::sync::broadcast;

impl ConnectionPool {
    pub async fn create_artists(
        &self,
        artists: &Vec<UserCreatedArtist>,
        current_user_id: i32,
    ) -> Result<Vec<Artist>> {
        let mut tx = self.pool.begin().await?;

        let mut created_artists = Vec::new();

        for artist in artists {
            let artist = sqlx::query_as!(
                Artist,
                r#"
                INSERT INTO artists (name, aliases, description, pictures, external_links, created_by_id)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (name) DO UPDATE SET
                    -- This is a no-op update that still triggers RETURNING
                    name = EXCLUDED.name
                RETURNING id, name, aliases, created_at, created_by_id, description, pictures, external_links, title_groups_amount, edition_groups_amount, torrents_amount, seeders_amount, leechers_amount, snatches_amount, total_size
                "#,
                artist.name,
                &artist.aliases,
                artist.description,
                &artist.pictures,
                &artist.external_links,
                current_user_id
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(Error::CouldNotCreateArtist)?;

            created_artists.push(artist);
        }

        tx.commit().await?;

        Ok(created_artists)
    }

    pub async fn create_artists_affiliation(
        &self,
        artists: &Vec<UserCreatedAffiliatedArtist>,
        current_user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<Vec<AffiliatedArtistHierarchy>> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        let values: Vec<String> = (0..artists.len())
            .map(|i| {
                format!(
                    "(${}, ${}, ${}::artist_role_enum[], ${}, ${})",
                    i * 5 + 1,
                    i * 5 + 2,
                    i * 5 + 3,
                    i * 5 + 4,
                    i * 5 + 5
                )
            })
            .collect();

        let insert_query = format!(
            "INSERT INTO affiliated_artists (title_group_id, artist_id, roles, nickname, created_by_id) VALUES {} RETURNING id, title_group_id, artist_id, roles, nickname, created_at, created_by_id",
            values.join(", ")
        );

        let mut q_insert = sqlx::query_as::<_, AffiliatedArtist>(sqlx::AssertSqlSafe(insert_query));
        for artist in artists {
            q_insert = q_insert
                .bind(artist.title_group_id)
                .bind(artist.artist_id)
                .bind(&artist.roles)
                .bind(
                    artist
                        .nickname
                        .clone()
                        .map(|nick| if nick.is_empty() { None } else { Some(nick) }),
                )
                .bind(current_user_id);
        }

        let created_affiliations = q_insert.fetch_all(&mut *tx).await.map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e
                && db_err.code().as_deref() == Some("23505")
            {
                return Error::DuplicateArtistAffiliation;
            }
            Error::CouldNotCreateArtistAffiliation(e)
        })?;

        // Update title_groups_amount for each affected artist
        for affiliation in &created_affiliations {
            sqlx::query!(
                r#"
                UPDATE artists
                SET title_groups_amount = title_groups_amount + 1
                WHERE id = $1
                "#,
                affiliation.artist_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        // one notification per title group, whatever the amount of subscribed artists affiliated to it
        let mut artist_ids_per_title_group: HashMap<i32, Vec<i64>> = HashMap::new();
        for affiliation in &created_affiliations {
            artist_ids_per_title_group
                .entry(affiliation.title_group_id)
                .or_default()
                .push(affiliation.artist_id);
        }

        let mut notified_user_ids: Vec<i32> = Vec::new();
        for (title_group_id, notified_artist_ids) in &artist_ids_per_title_group {
            let user_ids = Self::notify_users_artist_title_groups(
                &mut tx,
                notified_artist_ids,
                *title_group_id,
                current_user_id,
            )
            .await?;
            notified_user_ids.extend(user_ids);
        }

        let artist_ids: Vec<i64> = created_affiliations
            .iter()
            .map(|aff| aff.artist_id)
            .collect();

        let fetched_artists: Vec<Artist> = sqlx::query_as!(
            Artist,
            r#"
        SELECT id, name, aliases, created_at, created_by_id, description, pictures, external_links, title_groups_amount, edition_groups_amount, torrents_amount, seeders_amount, leechers_amount, snatches_amount, total_size FROM artists WHERE id = ANY($1)
        "#,
            &artist_ids
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateArtistAffiliation)?;

        tx.commit().await?;

        if !notified_user_ids.is_empty() {
            let _ = notification_sender.send(NotificationEvent::ArtistTitleGroup {
                user_ids: notified_user_ids,
            });
        }

        let mut affiliated_artist_hierarchies: Vec<AffiliatedArtistHierarchy> = Vec::new();

        for affiliation in created_affiliations {
            if let Some(artist) = fetched_artists
                .iter()
                .find(|a| a.id == affiliation.artist_id)
            {
                affiliated_artist_hierarchies.push(AffiliatedArtistHierarchy {
                    id: affiliation.id,
                    title_group_id: affiliation.title_group_id,
                    artist_id: affiliation.artist_id,
                    roles: affiliation.roles,
                    nickname: affiliation.nickname,
                    created_at: affiliation.created_at,
                    created_by_id: affiliation.created_by_id,
                    artist: artist.clone(),
                });
            }
        }

        Ok(affiliated_artist_hierarchies)
    }

    pub async fn find_artists_lite(&self, name: &String, limit: i64) -> Result<Vec<ArtistLite>> {
        let found_artists = sqlx::query_as!(
            ArtistLite,
            r#"
            SELECT name, id, aliases, pictures
            FROM artists
            WHERE unaccent(name) ILIKE '%' || unaccent($1) || '%'
               OR EXISTS (
                   SELECT 1 FROM unnest(aliases) AS alias
                   WHERE unaccent(alias) ILIKE '%' || unaccent($1) || '%'
               )
            LIMIT $2
        "#,
            name,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotSearchForArtists)?;

        Ok(found_artists)
    }

    pub async fn search_artists(
        &self,
        form: &SearchArtistsQuery,
    ) -> Result<PaginatedResults<ArtistSearchResult>> {
        let offset = (form.page - 1) * form.page_size;

        let total_items: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM artists
            WHERE $1::TEXT IS NULL
               OR unaccent(name) ILIKE '%' || unaccent($1) || '%'
               OR EXISTS (
                   SELECT 1 FROM unnest(aliases) AS alias
                   WHERE unaccent(alias) ILIKE '%' || unaccent($1) || '%'
               )
            "#,
            form.name,
        )
        .fetch_one(&self.pool)
        .await
        .unwrap()
        .unwrap();

        let results = sqlx::query_as!(
            ArtistSearchResult,
            r#"
            SELECT id, name, aliases, created_at, created_by_id, pictures, title_groups_amount
            FROM artists
            WHERE $1::TEXT IS NULL
               OR unaccent(name) ILIKE '%' || unaccent($1) || '%'
               OR EXISTS (
                   SELECT 1 FROM unnest(aliases) AS alias
                   WHERE unaccent(alias) ILIKE '%' || unaccent($1) || '%'
               )
            ORDER BY
                CASE WHEN $4 = 'name' AND $5 = 'asc' THEN name END ASC,
                CASE WHEN $4 = 'name' AND $5 = 'desc' THEN name END DESC,
                CASE WHEN $4 = 'created_at' AND $5 = 'asc' THEN created_at END ASC,
                CASE WHEN $4 = 'created_at' AND $5 = 'desc' THEN created_at END DESC,
                CASE WHEN $4 = 'title_groups_amount' AND $5 = 'asc' THEN title_groups_amount END ASC,
                CASE WHEN $4 = 'title_groups_amount' AND $5 = 'desc' THEN title_groups_amount END DESC
            OFFSET $2 LIMIT $3
            "#,
            form.name,
            offset as i64,
            form.page_size as i64,
            form.order_by_column.to_string(),
            form.order_by_direction.to_string()
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

    pub async fn find_artist_by_id(&self, artist_id: i64) -> Result<Artist> {
        sqlx::query_as!(
            Artist,
            r#"
                SELECT id, name, aliases, created_at, created_by_id, description, pictures, external_links, title_groups_amount, edition_groups_amount, torrents_amount, seeders_amount, leechers_amount, snatches_amount, total_size
                FROM artists
                WHERE id = $1;
            "#,
            artist_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindArtist)
    }

    pub async fn find_artist_enriched(
        &self,
        artist_id: i64,
        current_user_id: i32,
    ) -> Result<ArtistEnriched> {
        let row = sqlx::query!(
            r#"
                SELECT
                    to_jsonb(a) AS "artist!: sqlx::types::Json<Artist>",
                    COALESCE((
                        SELECT jsonb_object_agg(tags.name, tags.cnt)
                        FROM (
                            SELECT tgt.name AS name, COUNT(*) AS cnt
                            FROM affiliated_artists aa
                            JOIN title_group_applied_tags tgat ON tgat.title_group_id = aa.title_group_id
                            JOIN title_group_tags tgt ON tgt.id = tgat.tag_id
                            WHERE aa.artist_id = a.id AND tgt.deleted_at IS NULL
                            GROUP BY tgt.name
                        ) tags
                    ), '{}'::jsonb) AS "tags!: sqlx::types::Json<HashMap<String, i64>>",
                    COALESCE((
                        SELECT jsonb_agg(
                            jsonb_build_object(
                                'forum_thread_id', art.forum_thread_id,
                                'thread_name', ft.name,
                                'created_at', art.created_at
                            )
                            ORDER BY art.created_at DESC
                        )
                        FROM artist_related_threads art
                        JOIN forum_threads ft ON ft.id = art.forum_thread_id
                        WHERE art.artist_id = a.id
                    ), '[]'::jsonb) AS "related_threads!: sqlx::types::Json<Vec<RelatedForumThread>>",
                    EXISTS (
                        SELECT 1
                        FROM subscriptions_artist_title_groups s
                        WHERE s.artist_id = a.id AND s.user_id = $2
                    ) AS "is_subscribed_to_title_groups!"
                FROM artists a
                WHERE a.id = $1
            "#,
            artist_id,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindArtist)?;

        let title_group_stats = self
            .get_title_group_catalog_stats(CatalogKind::Artist, artist_id)
            .await?;

        Ok(ArtistEnriched {
            artist: row.artist.0,
            is_subscribed_to_title_groups: row.is_subscribed_to_title_groups,
            tags: row.tags.0,
            related_threads: row.related_threads.0,
            title_group_stats,
        })
    }

    pub async fn update_artist_data(&self, updated_artist: &EditedArtist) -> Result<Artist> {
        sqlx::query_as!(
            Artist,
            r#"
                UPDATE artists
                SET name = $1, aliases = $2, description = $3, pictures = $4, external_links = $5
                WHERE id = $6
                RETURNING id, name, aliases, created_at, created_by_id, description, pictures, external_links, title_groups_amount, edition_groups_amount, torrents_amount, seeders_amount, leechers_amount, snatches_amount, total_size
            "#,
            updated_artist.name,
            &updated_artist.aliases,
            updated_artist.description,
            &updated_artist.pictures,
            &updated_artist.external_links,
            updated_artist.id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotUpdateArtist)
    }

    pub async fn update_artist_pictures(&self, artist_id: i64, pictures: &[String]) -> Result<()> {
        sqlx::query!(
            r#"UPDATE artists SET pictures = $1 WHERE id = $2"#,
            pictures,
            artist_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotUpdateArtist)?;
        Ok(())
    }

    pub async fn find_title_group_ids_of_artists_affiliation(
        &self,
        affiliation_ids: &[i64],
    ) -> Result<Vec<i32>> {
        Ok(sqlx::query_scalar!(
            "SELECT DISTINCT title_group_id FROM affiliated_artists WHERE id = ANY($1)",
            affiliation_ids
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn delete_artists_affiliation(&self, affiliation_ids: &Vec<i64>) -> Result<()> {
        // Get artist_id for affiliations being deleted
        let affected_affiliations: Vec<(i64,)> = sqlx::query_as(
            r#"
            SELECT artist_id
            FROM affiliated_artists
            WHERE id = ANY($1)
            "#,
        )
        .bind(affiliation_ids)
        .fetch_all(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            DELETE FROM affiliated_artists
            WHERE id = ANY($1)
            "#,
            &affiliation_ids
        )
        .execute(&self.pool)
        .await?;

        // Update title_groups_amount for each affected artist
        for (artist_id,) in affected_affiliations {
            sqlx::query!(
                r#"
                UPDATE artists
                SET title_groups_amount = title_groups_amount - 1
                WHERE id = $1
                "#,
                artist_id
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Aggregates `seeders`, `leechers` and `times_completed` from all torrents
    /// affiliated with each artist and stores the sums in the artists table.
    /// Returns the number of artist rows that were actually changed.
    pub async fn update_artist_peer_stats(&self) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            WITH artist_stats AS (
                SELECT aa.artist_id,
                       COALESCE(SUM(t.seeders), 0)::INT AS seeders,
                       COALESCE(SUM(t.leechers), 0)::INT AS leechers,
                       COALESCE(SUM(t.times_completed), 0)::INT AS snatches,
                       COALESCE(SUM(t.size), 0)::BIGINT AS total_size
                FROM affiliated_artists aa
                JOIN edition_groups eg ON eg.title_group_id = aa.title_group_id
                JOIN torrents t ON t.edition_group_id = eg.id AND t.deleted_at IS NULL
                GROUP BY aa.artist_id
            )
            UPDATE artists a
            SET seeders_amount = COALESCE(s.seeders, 0),
                leechers_amount = COALESCE(s.leechers, 0),
                snatches_amount = COALESCE(s.snatches, 0),
                total_size = COALESCE(s.total_size, 0)
            FROM artists a2
            LEFT JOIN artist_stats s ON s.artist_id = a2.id
            WHERE a.id = a2.id
              AND (a.seeders_amount != COALESCE(s.seeders, 0)
                OR a.leechers_amount != COALESCE(s.leechers, 0)
                OR a.snatches_amount != COALESCE(s.snatches, 0)
                OR a.total_size != COALESCE(s.total_size, 0))
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn delete_artist(&self, artist_id: i64) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM artists
            WHERE id = $1
            "#,
            artist_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteArtist)?;

        Ok(())
    }

    /// For title groups with at most 2 affiliated artists, returns them;
    /// for title groups with more, returns a single dummy artist (id 0, empty name)
    pub async fn find_affiliated_artists_lite_by_title_group_ids(
        &self,
        title_group_ids: &[i32],
    ) -> Result<HashMap<i32, Vec<AffiliatedArtistLite>>> {
        if title_group_ids.is_empty() {
            return Ok(HashMap::new());
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
            title_group_ids
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

        Ok(grouped_artists)
    }
}
