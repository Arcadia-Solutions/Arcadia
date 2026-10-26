use crate::{
    connection_pool::ConnectionPool,
    models::edition_group::{EditedEditionGroup, EditionGroup, UserCreatedEditionGroup},
};
use arcadia_common::error::{Error, Result};

impl ConnectionPool {
    pub async fn create_edition_group(
        &self,
        edition_group_form: &UserCreatedEditionGroup,
        current_user_id: i32,
    ) -> Result<EditionGroup> {
        const CREATE_EDITION_GROUPS_QUERY: &str = r#"
            INSERT INTO edition_groups (title_group_id, name, release_date, release_date_only_year_known, created_by_id, description, distributor, covers, external_links, source, additional_information)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::source_enum, $11)
            RETURNING id, title_group_id, name, release_date, release_date_only_year_known, created_at, updated_at, created_by_id, description, distributor, covers, external_links, source, additional_information;
        "#;

        let created_edition_group = sqlx::query_as::<_, EditionGroup>(CREATE_EDITION_GROUPS_QUERY)
            .bind(edition_group_form.title_group_id)
            .bind(&edition_group_form.name)
            .bind(edition_group_form.release_date)
            .bind(edition_group_form.release_date_only_year_known)
            .bind(current_user_id)
            .bind(&edition_group_form.description)
            .bind(&edition_group_form.distributor)
            .bind(&edition_group_form.covers)
            .bind(&edition_group_form.external_links)
            .bind(&edition_group_form.source)
            .bind(&edition_group_form.additional_information)
            .fetch_one(&self.pool)
            .await
            .map_err(Error::CouldNotCreateEditionGroup)?;

        // Update edition_groups_amount for all affiliated artists of this title group
        sqlx::query!(
            r#"
            UPDATE artists
            SET edition_groups_amount = edition_groups_amount + 1
            WHERE id IN (
                SELECT DISTINCT artist_id
                FROM affiliated_artists
                WHERE title_group_id = $1
            )
            "#,
            edition_group_form.title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Increment user's edition_groups counter
        sqlx::query!(
            r#"
            UPDATE users
            SET edition_groups = edition_groups + 1
            WHERE id = $1
            "#,
            current_user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(created_edition_group)
    }

    pub async fn find_edition_group(&self, edition_group_id: i32) -> Result<EditionGroup> {
        let edition_group = sqlx::query_as!(
            EditionGroup,
            r#"
            SELECT
                id, title_group_id, name, release_date, release_date_only_year_known,
                created_at, updated_at, created_by_id, description,
                distributor, covers AS "covers!: _", external_links AS "external_links!: _",
                source AS "source: _", additional_information
            FROM edition_groups
            WHERE id = $1
            "#,
            edition_group_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::EditionGroupNotFound)?;

        Ok(edition_group)
    }

    pub async fn delete_edition_group(&self, edition_group_id: i32) -> Result<()> {
        // Check if there are any undeleted torrents linked to this edition group
        let has_undeleted_torrents: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM torrents
                WHERE edition_group_id = $1 AND deleted_at IS NULL
            ) AS "exists!"
            "#,
            edition_group_id
        )
        .fetch_one(&self.pool)
        .await?;

        if has_undeleted_torrents {
            return Err(Error::EditionGroupHasUndeletedTorrents);
        }

        let edition_group = self.find_edition_group(edition_group_id).await?;

        // Decrement edition_groups_amount for all affiliated artists of this title group
        sqlx::query!(
            r#"
            UPDATE artists
            SET edition_groups_amount = edition_groups_amount - 1
            WHERE id IN (
                SELECT DISTINCT artist_id
                FROM affiliated_artists
                WHERE title_group_id = $1
            )
            "#,
            edition_group.title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Decrement user's edition_groups counter
        sqlx::query!(
            r#"
            UPDATE users
            SET edition_groups = edition_groups - 1
            WHERE id = $1
            "#,
            edition_group.created_by_id
        )
        .execute(&self.pool)
        .await?;

        // Delete the edition group
        sqlx::query!(
            r#"
            DELETE FROM edition_groups WHERE id = $1
            "#,
            edition_group_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_edition_group(
        &self,
        edited_edition_group: &EditedEditionGroup,
    ) -> Result<EditionGroup> {
        let updated_edition_group = sqlx::query_as!(
            EditionGroup,
            r#"
            UPDATE edition_groups
            SET
                name = $2,
                release_date = $3,
                release_date_only_year_known = $4,
                description = $5,
                distributor = $6,
                covers = $7,
                external_links = $8,
                source = $9::source_enum,
                additional_information = $10,
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, title_group_id, name, release_date, release_date_only_year_known,
                created_at, updated_at, created_by_id, description,
                distributor, covers AS "covers!: _", external_links AS "external_links!: _",
                source AS "source: _", additional_information
            "#,
            edited_edition_group.id,
            edited_edition_group.name,
            edited_edition_group.release_date,
            edited_edition_group.release_date_only_year_known,
            edited_edition_group.description,
            edited_edition_group.distributor,
            edited_edition_group.covers.as_slice(),
            edited_edition_group.external_links.as_slice(),
            edited_edition_group.source as _,
            edited_edition_group.additional_information
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::ErrorWhileUpdatingEditionGroup(e.to_string()))?;

        Ok(updated_edition_group)
    }
}
