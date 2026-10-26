use crate::{
    connection_pool::ConnectionPool,
    models::{
        edition_group::Source,
        title_group::{ContentType, TitleGroupCategory},
        torrent::{Language, TorrentSearch, VideoResolution},
        user_badge::{
            EditedUserBadge, EditedUserBadgeCategory, UserBadge, UserBadgeCategory,
            UserBadgeListItem, UserBadgeType, UserCreatedUserBadge, UserCreatedUserBadgeCategory,
            UserEarnedBadge, UserEarnedBadgeWithDetails,
        },
    },
    utils::{tag_expression::parse_tag_expression, validate_badge_criteria_shape},
};
use arcadia_common::{
    error::{Error, Result},
    services::torrent_service::looks_like_url,
};

impl ConnectionPool {
    pub async fn create_user_badge_category(
        &self,
        category: &UserCreatedUserBadgeCategory,
        current_user_id: i32,
    ) -> Result<UserBadgeCategory> {
        if category.name.trim().is_empty() {
            return Err(Error::UserBadgeCategoryNameEmpty);
        }

        let created = sqlx::query_as!(
            UserBadgeCategory,
            r#"
                INSERT INTO user_badge_categories (name, created_by_id)
                VALUES ($1, $2)
                RETURNING id, name, created_at, created_by_id
            "#,
            category.name,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateUserBadgeCategory)?;

        Ok(created)
    }

    pub async fn update_user_badge_category(
        &self,
        edited: &EditedUserBadgeCategory,
    ) -> Result<UserBadgeCategory> {
        if edited.name.trim().is_empty() {
            return Err(Error::UserBadgeCategoryNameEmpty);
        }

        let updated = sqlx::query_as!(
            UserBadgeCategory,
            r#"
                UPDATE user_badge_categories
                SET name = $1
                WHERE id = $2
                RETURNING id, name, created_at, created_by_id
            "#,
            edited.name,
            edited.id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::UserBadgeCategoryNotFound,
            _ => Error::CouldNotUpdateUserBadgeCategory(e),
        })?;

        Ok(updated)
    }

    pub async fn delete_user_badge_category(&self, category_id: i32) -> Result<()> {
        let badge_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM user_badges WHERE category_id = $1"#,
            category_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteUserBadgeCategory)?
        .unwrap_or(0);

        if badge_count > 0 {
            return Err(Error::UserBadgeCategoryHasBadges);
        }

        let result = sqlx::query!(
            r#"DELETE FROM user_badge_categories WHERE id = $1"#,
            category_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteUserBadgeCategory)?;

        if result.rows_affected() == 0 {
            return Err(Error::UserBadgeCategoryNotFound);
        }

        Ok(())
    }

    pub async fn find_all_user_badge_categories(&self) -> Result<Vec<UserBadgeCategory>> {
        let categories = sqlx::query_as!(
            UserBadgeCategory,
            r#"
                SELECT id, name, created_at, created_by_id
                FROM user_badge_categories
                ORDER BY name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindUserBadgeCategories)?;

        Ok(categories)
    }

    pub async fn create_user_badge(
        &self,
        badge: &UserCreatedUserBadge,
        current_user_id: i32,
    ) -> Result<UserBadge> {
        if badge.name.trim().is_empty() {
            return Err(Error::UserBadgeNameEmpty);
        }
        validate_badge_criteria_shape(&badge.badge_type, badge.criteria.as_ref())?;

        let created = sqlx::query_as!(
            UserBadge,
            r#"
                INSERT INTO user_badges (
                    name, description, image_url, category_id, badge_type,
                    is_secret, revoke_when_criteria_unmet, criteria, created_by_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING
                    id, name, description, image_url, category_id,
                    badge_type AS "badge_type: UserBadgeType",
                    is_secret, revoke_when_criteria_unmet, criteria,
                    created_at, created_by_id
            "#,
            badge.name,
            badge.description,
            badge.image_url,
            badge.category_id,
            &badge.badge_type as &UserBadgeType,
            badge.is_secret,
            badge.revoke_when_criteria_unmet,
            badge.criteria.as_ref(),
            current_user_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateUserBadge)?;

        Ok(created)
    }

    pub async fn update_user_badge(&self, edited: &EditedUserBadge) -> Result<UserBadge> {
        if edited.name.trim().is_empty() {
            return Err(Error::UserBadgeNameEmpty);
        }
        validate_badge_criteria_shape(&edited.badge_type, edited.criteria.as_ref())?;

        let updated = sqlx::query_as!(
            UserBadge,
            r#"
                UPDATE user_badges
                SET name = $1, description = $2, image_url = $3, category_id = $4,
                    badge_type = $5, is_secret = $6, revoke_when_criteria_unmet = $7,
                    criteria = $8
                WHERE id = $9
                RETURNING
                    id, name, description, image_url, category_id,
                    badge_type AS "badge_type: UserBadgeType",
                    is_secret, revoke_when_criteria_unmet, criteria,
                    created_at, created_by_id
            "#,
            edited.name,
            edited.description,
            edited.image_url,
            edited.category_id,
            &edited.badge_type as &UserBadgeType,
            edited.is_secret,
            edited.revoke_when_criteria_unmet,
            edited.criteria.as_ref(),
            edited.id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::UserBadgeNotFound,
            _ => Error::CouldNotUpdateUserBadge(e),
        })?;

        Ok(updated)
    }

    pub async fn delete_user_badge(&self, badge_id: i32) -> Result<()> {
        let result = sqlx::query!(r#"DELETE FROM user_badges WHERE id = $1"#, badge_id)
            .execute(&self.pool)
            .await
            .map_err(Error::CouldNotDeleteUserBadge)?;

        if result.rows_affected() == 0 {
            return Err(Error::UserBadgeNotFound);
        }

        Ok(())
    }

    pub async fn find_all_user_badges(
        &self,
        viewer_can_see_secret: bool,
    ) -> Result<Vec<UserBadgeListItem>> {
        let badges = sqlx::query_as!(
            UserBadge,
            r#"
                SELECT
                    id, name, description, image_url, category_id,
                    badge_type AS "badge_type: UserBadgeType",
                    is_secret, revoke_when_criteria_unmet, criteria,
                    created_at, created_by_id
                FROM user_badges
                ORDER BY name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindUserBadges)?;

        let items = badges
            .into_iter()
            .map(|b| {
                if b.is_secret && !viewer_can_see_secret {
                    UserBadgeListItem::Hidden {
                        id: b.id,
                        is_secret: true,
                    }
                } else {
                    UserBadgeListItem::Visible(b)
                }
            })
            .collect();

        Ok(items)
    }

    pub async fn find_all_user_badges_full(&self) -> Result<Vec<UserBadge>> {
        let badges = sqlx::query_as!(
            UserBadge,
            r#"
                SELECT
                    id, name, description, image_url, category_id,
                    badge_type AS "badge_type: UserBadgeType",
                    is_secret, revoke_when_criteria_unmet, criteria,
                    created_at, created_by_id
                FROM user_badges
                ORDER BY id ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindUserBadges)?;

        Ok(badges)
    }

    pub async fn find_user_earned_badges(
        &self,
        user_id: i32,
    ) -> Result<Vec<UserEarnedBadgeWithDetails>> {
        let rows = sqlx::query_as!(
            UserEarnedBadgeWithDetails,
            r#"
                SELECT
                    ueb.id, ueb.user_id, ueb.badge_id, ueb.awarded_at,
                    ueb.awarded_by_id, ueb.note,
                    ub.name AS badge_name,
                    ub.description AS badge_description,
                    ub.image_url AS badge_image_url,
                    ub.category_id AS badge_category_id,
                    ub.badge_type AS "badge_type: UserBadgeType",
                    ub.is_secret AS badge_is_secret
                FROM user_earned_badges ueb
                JOIN user_badges ub ON ub.id = ueb.badge_id
                WHERE ueb.user_id = $1
                ORDER BY ueb.awarded_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindUserBadges)?;

        Ok(rows)
    }

    pub async fn award_user_badge(
        &self,
        user_id: i32,
        badge_id: i32,
        awarded_by_id: Option<i32>,
        note: Option<&str>,
    ) -> Result<UserEarnedBadge> {
        let inserted = sqlx::query_as!(
            UserEarnedBadge,
            r#"
                INSERT INTO user_earned_badges (user_id, badge_id, awarded_by_id, note)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (user_id, badge_id) DO NOTHING
                RETURNING id, user_id, badge_id, awarded_at, awarded_by_id, note
            "#,
            user_id,
            badge_id,
            awarded_by_id,
            note,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::CouldNotAwardUserBadge)?;

        match inserted {
            Some(row) => Ok(row),
            None => Err(Error::UserAlreadyHasBadge),
        }
    }

    pub async fn revoke_user_earned_badge(&self, user_earned_badge_id: i32) -> Result<()> {
        let result = sqlx::query!(
            r#"DELETE FROM user_earned_badges WHERE id = $1"#,
            user_earned_badge_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotRevokeUserBadge)?;

        if result.rows_affected() == 0 {
            return Err(Error::UserEarnedBadgeNotFound);
        }

        Ok(())
    }

    pub async fn revoke_auto_earned_badge(&self, user_id: i32, badge_id: i32) -> Result<bool> {
        let result = sqlx::query!(
            r#"
                DELETE FROM user_earned_badges
                WHERE user_id = $1 AND badge_id = $2 AND awarded_by_id IS NULL
            "#,
            user_id,
            badge_id,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotRevokeUserBadge)?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_user_ids_with_badge(&self, badge_id: i32) -> Result<Vec<(i32, Option<i32>)>> {
        let rows = sqlx::query!(
            r#"
                SELECT user_id, awarded_by_id
                FROM user_earned_badges
                WHERE badge_id = $1
            "#,
            badge_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindUserBadges)?;

        Ok(rows
            .into_iter()
            .map(|row| (row.user_id, row.awarded_by_id))
            .collect())
    }

    pub async fn find_qualifying_uploader_ids(
        &self,
        form: &TorrentSearch,
        minimum_title_group_amount: i64,
    ) -> Result<Vec<i32>> {
        let (name_filter, external_link_filter) = match &form.title_group_name {
            Some(s) => {
                let input = s.trim();
                if input.is_empty() {
                    (None, None)
                } else if looks_like_url(input) {
                    (None, Some(input.trim_end_matches('/').to_string()))
                } else {
                    (Some(input.to_string()), None)
                }
            }
            None => (None, None),
        };

        let tag_filter_jsonb: Option<serde_json::Value> = match &form.title_group_tags {
            Some(s) => parse_tag_expression(s).map_err(Error::InvalidTagExpression)?,
            None => None,
        };

        let ids = sqlx::query_scalar!(
            r#"
            SELECT torrent_created_by_id AS "user_id!"
            FROM title_group_hierarchy_lite tgh
            WHERE tgh.torrent_created_by_id IS NOT NULL
              AND EXISTS (
                SELECT 1 FROM users u
                WHERE u.id = tgh.torrent_created_by_id AND u.banned = FALSE
              )
              AND ($1::BOOLEAN IS NULL OR tgh.torrent_staff_checked = $1)
              AND ($2::BOOLEAN IS NULL OR tgh.torrent_reported = $2)
              AND ($17::BOOLEAN IS NULL OR (NULLIF(tgh.torrent_trumpable, '') IS NOT NULL) = $17)
              AND (
                $3::BIGINT IS NULL OR
                EXISTS (SELECT 1 FROM affiliated_artists aa WHERE aa.title_group_id = tgh.title_group_id AND aa.artist_id = $3)
              )
              AND (
                $4::TEXT IS NULL OR
                tgh.title_group_name ILIKE '%' || $4 || '%' ESCAPE '\' OR
                tgh.title_group_series_name ILIKE '%' || $4 || '%' ESCAPE '\' OR
                EXISTS (SELECT 1 FROM unnest(tgh.title_group_name_aliases) alias WHERE alias ILIKE '%' || $4 || '%')
              )
              AND (
                $5::TEXT IS NULL
                OR EXISTS (
                    SELECT 1 FROM unnest(tgh.title_group_external_links) link
                    WHERE rtrim(link, '/') = $5
                )
              )
              AND ($6::BOOLEAN IS TRUE OR tgh.torrent_id IS NOT NULL)
              AND ($7::BIGINT IS NULL OR tgh.title_group_series_id = $7)
              AND ($8::INT IS NULL OR
                EXISTS (SELECT 1 FROM collage_entry ce WHERE ce.title_group_id = tgh.title_group_id AND ce.collage_id = $8)
              )
              AND (CARDINALITY($9::content_type_enum[]) = 0 OR tgh.title_group_content_type = ANY($9))
              AND (CARDINALITY($10::title_group_category_enum[]) = 0 OR tgh.title_group_category = ANY($10))
              AND (CARDINALITY($11::source_enum[]) = 0 OR tgh.edition_group_source = ANY($11))
              AND (CARDINALITY($12::video_resolution_enum[]) = 0 OR tgh.torrent_video_resolution = ANY($12))
              AND (CARDINALITY($13::language_enum[]) = 0 OR tgh.torrent_languages && $13)
              AND (
                $14::INT IS NULL OR
                EXISTS (
                    SELECT 1 FROM torrent_activities ta
                    WHERE ta.torrent_id = tgh.torrent_id
                      AND ta.user_id = $14
                      AND ta.completed_at IS NOT NULL
                )
              )
              AND (
                $15::JSONB IS NULL OR
                EXISTS (
                    SELECT 1 FROM jsonb_array_elements($15) AS clause
                    WHERE COALESCE(ARRAY(SELECT jsonb_array_elements_text(clause->'include'))::varchar[], '{}') <@ title_group_tag_names
                    AND NOT title_group_tag_names && COALESCE(ARRAY(SELECT jsonb_array_elements_text(clause->'exclude'))::varchar[], '{}')
                )
              )
            GROUP BY torrent_created_by_id
            HAVING COUNT(DISTINCT title_group_id) >= $16
            "#,
            form.torrent_staff_checked,
            form.torrent_reported,
            form.artist_id,
            name_filter,
            external_link_filter,
            form.title_group_include_empty_groups,
            form.series_id,
            form.collage_id,
            form.title_group_content_type.as_slice() as &[ContentType],
            form.title_group_category.as_slice() as &[TitleGroupCategory],
            form.edition_group_source.as_slice() as &[Source],
            form.torrent_video_resolution.as_slice() as &[VideoResolution],
            form.torrent_language.as_slice() as &[Language],
            form.torrent_snatched_by_id,
            tag_filter_jsonb,
            minimum_title_group_amount,
            form.torrent_trumpable,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::ErrorSearchingForTorrents(e.to_string()))?;

        Ok(ids)
    }

    pub async fn find_qualifying_forum_post_user_ids(
        &self,
        minimum_post_character_count: i32,
        required_substring: Option<&str>,
        minimum_post_amount: i32,
    ) -> Result<Vec<i32>> {
        let ids = sqlx::query_scalar!(
            r#"
                SELECT fp.created_by_id AS "user_id!"
                FROM forum_posts fp
                JOIN users u ON u.id = fp.created_by_id AND u.banned = FALSE
                WHERE char_length(fp.content) >= $1
                  AND ($2::TEXT IS NULL OR fp.content ILIKE '%' || $2 || '%')
                GROUP BY fp.created_by_id
                HAVING COUNT(*) >= $3
            "#,
            minimum_post_character_count as i64,
            required_substring,
            i64::from(minimum_post_amount),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindUserBadges)?;

        Ok(ids)
    }

    pub async fn find_qualifying_forum_thread_user_ids(
        &self,
        minimum_thread_name_character_count: i32,
        required_substring: Option<&str>,
        minimum_thread_amount: i32,
    ) -> Result<Vec<i32>> {
        let ids = sqlx::query_scalar!(
            r#"
                SELECT ft.created_by_id AS "user_id!"
                FROM forum_threads ft
                JOIN users u ON u.id = ft.created_by_id AND u.banned = FALSE
                WHERE char_length(ft.name) >= $1
                  AND ($2::TEXT IS NULL OR ft.name ILIKE '%' || $2 || '%')
                GROUP BY ft.created_by_id
                HAVING COUNT(*) >= $3
            "#,
            minimum_thread_name_character_count as i64,
            required_substring,
            i64::from(minimum_thread_amount),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindUserBadges)?;

        Ok(ids)
    }
}
