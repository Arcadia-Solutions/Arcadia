use crate::{
    connection_pool::ConnectionPool,
    models::{
        arcadia_settings::DisplayableUserStats,
        bonus_points_log::BonusPointsLogAction,
        common::PaginatedResults,
        conversation::MassMessageRecipient,
        ip_check::{IpCheck, IpCheckProvider, UserRegistrationIp},
        user::{
            EditedUser, EditedUserClass, HideableUserList, PublicUser, SearchUsersQuery, UserClass,
            UserCreatedUserClass, UserCreatedUserWarning, UserLite, UserParanoiaSettings,
            UserPermission, UserPermissionMatchMode, UserSearchResult, UserSettings,
            UserSettingsResponse, UserWarning, UserWithStats,
        },
    },
};
use arcadia_common::error::{Error, Result};
use arcadia_shared::tracker::models::user::{APIInsertUser, APIUpdateUserMaxSnatchesPerDay};
use chrono::{DateTime, Utc};
use reqwest::Method;
use sqlx::types::ipnetwork::IpNetwork;

struct UnbannedUser {
    id: i32,
    passkey: String,
    max_snatches_per_day: Option<i32>,
}

impl ConnectionPool {
    pub async fn find_user_profile(&self, id: &i32) -> Result<PublicUser> {
        sqlx::query_as!(
            PublicUser,
            r#"
                SELECT
                    id,
                    username,
                    avatar,
                    created_at AS "created_at?",
                    description,
                    uploaded AS "uploaded?",
                    downloaded AS "downloaded?",
                    real_uploaded AS "real_uploaded?",
                    real_downloaded AS "real_downloaded?",
                    last_seen AS "last_seen?",
                    class_name,
                    class_locked,
                    title_groups AS "title_groups?",
                    edition_groups AS "edition_groups?",
                    torrents AS "torrents?",
                    forum_posts AS "forum_posts?",
                    forum_threads AS "forum_threads?",
                    title_group_comments AS "title_group_comments?",
                    request_comments AS "request_comments?",
                    artist_comments AS "artist_comments?",
                    seeding AS "seeding?",
                    leeching AS "leeching?",
                    snatched AS "snatched?",
                    seeding_size AS "seeding_size?",
                    requests_filled AS "requests_filled?",
                    collages_started AS "collages_started?",
                    requests_voted AS "requests_voted?",
                    average_seeding_time AS "average_seeding_time?",
                    invited AS "invited?",
                    invitations AS "invitations?",
                    bonus_points AS "bonus_points?",
                    warned,
                    banned,
                    custom_title,
                    paranoia_hidden_stats AS "paranoia_hidden_stats: Vec<DisplayableUserStats>",
                    paranoia_hidden_lists AS "paranoia_hidden_lists: Vec<HideableUserList>"
                FROM users
                WHERE id = $1
            "#,
            *id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => Error::UserWithIdNotFound(*id),
            error => Error::GenericDatabaseError(error),
        })
    }

    // this keeps track of when the user made a request for the last time
    // as well as increments the streak if needed
    pub async fn update_last_seen_and_streak(&self, id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET
                highest_streak = GREATEST(
                    highest_streak,
                    CASE
                        WHEN last_seen::date = CURRENT_DATE - INTERVAL '1 day' THEN current_streak + 1
                        WHEN last_seen::date < CURRENT_DATE - INTERVAL '1 day' THEN 1
                        ELSE current_streak
                    END
                ),
                current_streak = CASE
                    WHEN last_seen::date = CURRENT_DATE - INTERVAL '1 day' THEN current_streak + 1
                    WHEN last_seen::date < CURRENT_DATE - INTERVAL '1 day' THEN 1
                    ELSE current_streak
                END,
                last_seen = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_user(&self, user_id: i32, edited_user: &EditedUser) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                UPDATE users
                SET avatar = $2, description = $3, email = $4
                WHERE id = $1
            "#,
            user_id,
            edited_user.avatar,
            edited_user.description,
            edited_user.email
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_settings(&self, user_id: i32) -> Result<UserSettingsResponse> {
        let user_settings = sqlx::query!(
            r#"
                SELECT
                    css_sheet_name,
                    irc_site_embed_enabled,
                    show_upload_method,
                    paranoia_hidden_stats AS "paranoia_hidden_stats: Vec<DisplayableUserStats>",
                    paranoia_hidden_lists AS "paranoia_hidden_lists: Vec<HideableUserList>",
                    uploaded_torrents.anonymous AS "anonymous_uploaded_torrents!",
                    uploaded_torrents.non_anonymous AS "non_anonymous_uploaded_torrents!"
                FROM users
                LEFT JOIN LATERAL (
                    SELECT
                        COUNT(*) FILTER (WHERE uploaded_as_anonymous) AS anonymous,
                        COUNT(*) FILTER (WHERE NOT uploaded_as_anonymous) AS non_anonymous
                    FROM torrents
                    WHERE created_by_id = users.id
                ) AS uploaded_torrents ON TRUE
                WHERE users.id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::UserWithIdNotFound(user_id))?;

        Ok(UserSettingsResponse {
            settings: UserSettings {
                css_sheet_name: user_settings.css_sheet_name,
                irc_site_embed_enabled: user_settings.irc_site_embed_enabled,
                show_upload_method: user_settings.show_upload_method,
                paranoia_hidden_stats: user_settings.paranoia_hidden_stats,
                paranoia_hidden_lists: user_settings.paranoia_hidden_lists,
            },
            anonymous_uploaded_torrents: user_settings.anonymous_uploaded_torrents,
            non_anonymous_uploaded_torrents: user_settings.non_anonymous_uploaded_torrents,
        })
    }

    pub async fn update_user_settings(&self, user_id: i32, settings: &UserSettings) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                UPDATE users
                SET
                    css_sheet_name = $2,
                    irc_site_embed_enabled = $3,
                    show_upload_method = $4,
                    paranoia_hidden_stats = $5,
                    paranoia_hidden_lists = $6
                WHERE id = $1
            "#,
            user_id,
            settings.css_sheet_name,
            settings.irc_site_embed_enabled,
            settings.show_upload_method,
            &settings.paranoia_hidden_stats as &[DisplayableUserStats],
            &settings.paranoia_hidden_lists as &[HideableUserList]
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Marks every torrent uploaded by the user as anonymous or as not anonymous.
    pub async fn update_all_uploaded_torrents_anonymity(
        &self,
        user_id: i32,
        uploaded_as_anonymous: bool,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
                UPDATE torrents
                SET uploaded_as_anonymous = $2
                WHERE created_by_id = $1 AND uploaded_as_anonymous != $2
            "#,
            user_id,
            uploaded_as_anonymous
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Returns the paranoia settings of a user, to know what may be shown about them.
    pub async fn find_user_paranoia_settings(&self, user_id: i32) -> Result<UserParanoiaSettings> {
        sqlx::query_as!(
            UserParanoiaSettings,
            r#"
                SELECT
                    paranoia_hidden_stats AS "paranoia_hidden_stats: Vec<DisplayableUserStats>",
                    paranoia_hidden_lists AS "paranoia_hidden_lists: Vec<HideableUserList>"
                FROM users
                WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::UserWithIdNotFound(user_id))
    }

    pub async fn update_user_custom_title(
        &self,
        user_id: i32,
        custom_title: Option<&str>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE users
                SET custom_title = $2
                WHERE id = $1
            "#,
            user_id,
            custom_title
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_user_registration_ip_check(
        &self,
        user_id: i32,
        registration_ip_check: &IpCheck,
    ) -> Result<()> {
        let result = &registration_ip_check.result;
        sqlx::query!(
            r#"
                UPDATE users
                SET registration_ip_check_provider = $2,
                    registration_ip_country_code = $3,
                    registration_ip_country = $4,
                    registration_ip_region = $5,
                    registration_ip_city = $6,
                    registration_ip_latitude = $7,
                    registration_ip_longitude = $8,
                    registration_ip_asn = $9,
                    registration_ip_organization = $10,
                    registration_ip_is_vpn = $11,
                    registration_ip_is_proxy = $12,
                    registration_ip_is_tor = $13,
                    registration_ip_is_hosting = $14
                WHERE id = $1
            "#,
            user_id,
            registration_ip_check.provider as _,
            result.country_code,
            result.country,
            result.region,
            result.city,
            result.latitude,
            result.longitude,
            result.asn,
            result.organization,
            result.is_vpn,
            result.is_proxy,
            result.is_tor,
            result.is_hosting
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_user_registration_ip(&self, user_id: i32) -> Result<UserRegistrationIp> {
        sqlx::query_as!(
            UserRegistrationIp,
            r#"
                SELECT registered_from_ip,
                       registration_ip_check_provider AS "provider: IpCheckProvider",
                       registration_ip_country_code AS country_code,
                       registration_ip_country AS country,
                       registration_ip_region AS region,
                       registration_ip_city AS city,
                       registration_ip_latitude AS latitude,
                       registration_ip_longitude AS longitude,
                       registration_ip_asn AS asn,
                       registration_ip_organization AS organization,
                       registration_ip_is_vpn AS is_vpn,
                       registration_ip_is_proxy AS is_proxy,
                       registration_ip_is_tor AS is_tor,
                       registration_ip_is_hosting AS is_hosting
                FROM users
                WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::UserWithIdNotFound(user_id))
    }

    pub async fn find_users_without_registration_ip_check(&self) -> Result<Vec<(i32, IpNetwork)>> {
        let users = sqlx::query!(
            r#"
                SELECT id, registered_from_ip
                FROM users
                WHERE registration_ip_check_provider IS NULL
                ORDER BY id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users
            .into_iter()
            .map(|user| (user.id, user.registered_from_ip))
            .collect())
    }

    pub async fn create_user_warning(
        &self,
        current_user_id: i32,
        user_warning: &UserCreatedUserWarning,
    ) -> Result<UserWarning> {
        let mut tx = self.pool.begin().await?;

        let _ = sqlx::query!(
            r#"
                UPDATE users
                SET warned = true,
                banned = CASE
                    WHEN $2 IS TRUE THEN TRUE
                    ELSE banned
                END
                WHERE id = $1
            "#,
            user_warning.user_id,
            user_warning.ban
        )
        .execute(&mut *tx)
        .await?;

        let user_warning = sqlx::query_as!(
            UserWarning,
            r#"
                INSERT INTO user_warnings (user_id, expires_at, reason, created_by_id, ban)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, user_id, created_at, expires_at, reason, created_by_id, ban, removed_at, removed_by_id
            "#,
            user_warning.user_id,
            user_warning.expires_at,
            user_warning.reason,
            current_user_id,
            user_warning.ban
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateGift)?;

        tx.commit().await?;

        Ok(user_warning)
    }

    pub async fn find_user_warnings(&self, user_id: i32) -> Vec<UserWarning> {
        sqlx::query_as!(
            UserWarning,
            r#"
                SELECT id, user_id, created_at, expires_at, reason, created_by_id, ban, removed_at, removed_by_id FROM user_warnings
                WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .expect("failed to get user warnings")
    }

    /// Sets `warned = false` and `banned = false` (if the warning had `ban = true`)
    /// for users whose all active (non-expired) warnings have expired.
    /// Keeps the warning rows in the database for history.
    pub async fn clear_expired_warnings(&self) -> Result<u64> {
        let mut tx = self.pool.begin().await?;

        // Unban users whose ban-warnings have all expired
        let unbanned_users = sqlx::query_as!(
            UnbannedUser,
            r#"
            UPDATE users
            SET banned = false
            WHERE banned = true
            AND id IN (
                SELECT user_id FROM user_warnings
                WHERE ban = true AND removed_at IS NULL
                GROUP BY user_id
                HAVING COUNT(*) FILTER (WHERE expires_at IS NULL OR expires_at > NOW()) = 0
            )
            RETURNING id, passkey, max_snatches_per_day
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;

        // Set warned = false for users who have no active (non-expired) warnings
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET warned = false
            WHERE warned = true
            AND id NOT IN (
                SELECT DISTINCT user_id FROM user_warnings
                WHERE removed_at IS NULL
                AND (expires_at IS NULL OR expires_at > NOW())
            )
            "#,
        )
        .execute(&mut *tx)
        .await?;

        let cleared_count = result.rows_affected();

        tx.commit().await?;

        self.notify_tracker_users_unbanned(&unbanned_users).await;

        Ok(cleared_count)
    }

    /// Marks all active warnings for a user as removed by setting `removed_at`
    /// and `removed_by_id`, and clears the `warned` and `banned` flags on the
    /// user. Warning rows are kept.
    pub async fn remove_user_warnings(&self, user_id: i32, current_user_id: i32) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
                UPDATE user_warnings
                SET removed_at = NOW(), removed_by_id = $2
                WHERE user_id = $1
                AND removed_at IS NULL
            "#,
            user_id,
            current_user_id
        )
        .execute(&mut *tx)
        .await?;

        let unbanned_user = sqlx::query_as!(
            UnbannedUser,
            r#"
                UPDATE users
                SET warned = false, banned = false
                WHERE id = $1
                RETURNING id, passkey, max_snatches_per_day
            "#,
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        // tracker upsert is idempotent, so notifying a user that was only warned is harmless
        self.notify_tracker_users_unbanned(&[unbanned_user]).await;

        Ok(())
    }

    /// The tracker drops banned users' passkeys when it (re)loads its state,
    /// so unbanned users must be (re)inserted for their announces to be accepted.
    async fn notify_tracker_users_unbanned(&self, users: &[UnbannedUser]) {
        for user in users {
            let payload = APIInsertUser {
                id: user.id as u32,
                passkey: user.passkey.parse().expect("invalid passkey format"),
                max_snatches_per_day: user.max_snatches_per_day.map(|x| x as u32),
            };

            self.notify_tracker(
                Method::PUT,
                &["users"],
                Some(&payload),
                &format!("insert unbanned user {}", user.id),
            )
            .await;
        }
    }

    pub async fn is_user_banned(&self, user_id: i32) -> Result<bool> {
        let result = sqlx::query_scalar!("SELECT banned FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.pool)
            .await?;

        let Some(banned) = result else {
            return Ok(true);
        };

        Ok(banned)
    }

    pub async fn is_user_warned(&self, user_id: i32) -> Result<bool> {
        let result = sqlx::query_scalar!("SELECT warned FROM users WHERE id = $1", user_id)
            .fetch_optional(&self.pool)
            .await?;

        let Some(warned) = result else {
            return Ok(false);
        };

        Ok(warned)
    }

    pub async fn create_user_class(&self, user_class: &UserCreatedUserClass) -> Result<UserClass> {
        sqlx::query_as!(
            UserClass,
            r#"
                INSERT INTO user_classes (
                    name,
                    new_permissions,
                    automatic_promotion,
                    automatic_demotion,
                    promotion_allowed_while_warned,
                    previous_user_class,
                    required_account_age_in_days,
                    required_ratio,
                    required_torrent_uploads,
                    required_torrent_uploads_in_unique_title_groups,
                    required_uploaded,
                    required_torrent_snatched,
                    required_downloaded,
                    required_forum_posts,
                    required_forum_posts_in_unique_threads,
                    required_title_group_comments,
                    required_seeding_size,
                    max_snatches_per_day,
                    promotion_cost_bonus_points
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                RETURNING
                    name,
                    new_permissions as "new_permissions: Vec<UserPermission>",
                    max_snatches_per_day,
                    automatic_promotion,
                    automatic_demotion,
                    promotion_allowed_while_warned,
                    previous_user_class,
                    required_account_age_in_days,
                    required_ratio,
                    required_torrent_uploads,
                    required_torrent_uploads_in_unique_title_groups,
                    required_uploaded,
                    required_torrent_snatched,
                    required_downloaded,
                    required_forum_posts,
                    required_forum_posts_in_unique_threads,
                    required_title_group_comments,
                    required_seeding_size,
                    promotion_cost_bonus_points
            "#,
            user_class.name,
            &user_class.new_permissions as &[UserPermission],
            user_class.automatic_promotion,
            user_class.automatic_demotion,
            user_class.promotion_allowed_while_warned,
            user_class.previous_user_class,
            user_class.required_account_age_in_days,
            user_class.required_ratio,
            user_class.required_torrent_uploads,
            user_class.required_torrent_uploads_in_unique_title_groups,
            user_class.required_uploaded,
            user_class.required_torrent_snatched,
            user_class.required_downloaded,
            user_class.required_forum_posts,
            user_class.required_forum_posts_in_unique_threads,
            user_class.required_title_group_comments,
            user_class.required_seeding_size,
            user_class.max_snatches_per_day,
            user_class.promotion_cost_bonus_points
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e
                && db_err.code() == Some(std::borrow::Cow::Borrowed("23505"))
            {
                return Error::UserClassAlreadyExists;
            }
            Error::CouldNotCreateUserClass(e)
        })
    }

    pub async fn get_user_class_by_name(&self, name: &str) -> Result<UserClass> {
        sqlx::query_as!(
            UserClass,
            r#"
                SELECT
                    name,
                    new_permissions as "new_permissions: Vec<UserPermission>",
                    max_snatches_per_day,
                    automatic_promotion,
                    automatic_demotion,
                    promotion_allowed_while_warned,
                    previous_user_class,
                    required_account_age_in_days,
                    required_ratio,
                    required_torrent_uploads,
                    required_torrent_uploads_in_unique_title_groups,
                    required_uploaded,
                    required_torrent_snatched,
                    required_downloaded,
                    required_forum_posts,
                    required_forum_posts_in_unique_threads,
                    required_title_group_comments,
                    required_seeding_size,
                    promotion_cost_bonus_points
                FROM user_classes
                WHERE name = $1
            "#,
            name
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::UserClassNotFound(name.to_string()))
    }

    pub async fn get_all_user_classes(&self) -> Result<Vec<UserClass>> {
        sqlx::query_as!(
            UserClass,
            r#"
                WITH RECURSIVE class_chain AS (
                    SELECT name, 0 as position
                    FROM user_classes
                    WHERE previous_user_class IS NULL
                      AND name IN (
                          SELECT previous_user_class FROM user_classes
                          WHERE previous_user_class IS NOT NULL
                      )
                    UNION ALL
                    SELECT uc.name, cc.position + 1
                    FROM user_classes uc
                    JOIN class_chain cc ON uc.previous_user_class = cc.name
                )
                SELECT
                    uc.name,
                    uc.new_permissions as "new_permissions: Vec<UserPermission>",
                    uc.max_snatches_per_day,
                    uc.automatic_promotion,
                    uc.automatic_demotion,
                    uc.promotion_allowed_while_warned,
                    uc.previous_user_class,
                    uc.required_account_age_in_days,
                    uc.required_ratio,
                    uc.required_torrent_uploads,
                    uc.required_torrent_uploads_in_unique_title_groups,
                    uc.required_uploaded,
                    uc.required_torrent_snatched,
                    uc.required_downloaded,
                    uc.required_forum_posts,
                    uc.required_forum_posts_in_unique_threads,
                    uc.required_title_group_comments,
                    uc.required_seeding_size,
                    uc.promotion_cost_bonus_points
                FROM user_classes uc
                LEFT JOIN class_chain cc ON uc.name = cc.name
                ORDER BY cc.position IS NULL, cc.position, uc.name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::from)
    }

    pub async fn update_user_class(
        &self,
        old_name: &str,
        edited_class: &EditedUserClass,
    ) -> Result<UserClass> {
        // First fetch old class to get old permissions for propagation
        let old_class = self.get_user_class_by_name(old_name).await?;

        // Update the class definition
        let updated_class = sqlx::query_as!(
            UserClass,
            r#"
                UPDATE user_classes
                SET
                    name = $2,
                    new_permissions = $3,
                    automatic_promotion = $4,
                    automatic_demotion = $5,
                    promotion_allowed_while_warned = $6,
                    previous_user_class = $7,
                    required_account_age_in_days = $8,
                    required_ratio = $9,
                    required_torrent_uploads = $10,
                    required_torrent_uploads_in_unique_title_groups = $11,
                    required_uploaded = $12,
                    required_torrent_snatched = $13,
                    required_downloaded = $14,
                    required_forum_posts = $15,
                    required_forum_posts_in_unique_threads = $16,
                    required_title_group_comments = $17,
                    required_seeding_size = $18,
                    max_snatches_per_day = $19,
                    promotion_cost_bonus_points = $20
                WHERE name = $1
                RETURNING
                    name,
                    new_permissions as "new_permissions: Vec<UserPermission>",
                    automatic_promotion,
                    automatic_demotion,
                    promotion_allowed_while_warned,
                    previous_user_class,
                    required_account_age_in_days,
                    required_ratio,
                    required_torrent_uploads,
                    required_torrent_uploads_in_unique_title_groups,
                    required_uploaded,
                    required_torrent_snatched,
                    required_downloaded,
                    required_forum_posts,
                    required_forum_posts_in_unique_threads,
                    required_title_group_comments,
                    required_seeding_size,
                    max_snatches_per_day,
                    promotion_cost_bonus_points
            "#,
            old_name,
            edited_class.name,
            &edited_class.new_permissions as &[UserPermission],
            edited_class.automatic_promotion,
            edited_class.automatic_demotion,
            edited_class.promotion_allowed_while_warned,
            edited_class.previous_user_class,
            edited_class.required_account_age_in_days,
            edited_class.required_ratio,
            edited_class.required_torrent_uploads,
            edited_class.required_torrent_uploads_in_unique_title_groups,
            edited_class.required_uploaded,
            edited_class.required_torrent_snatched,
            edited_class.required_downloaded,
            edited_class.required_forum_posts,
            edited_class.required_forum_posts_in_unique_threads,
            edited_class.required_title_group_comments,
            edited_class.required_seeding_size,
            edited_class.max_snatches_per_day,
            edited_class.promotion_cost_bonus_points
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                Error::UserClassNotFound(old_name.to_string())
            } else if let sqlx::Error::Database(ref db_err) = e {
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    return Error::UserClassAlreadyExists;
                }
                Error::CouldNotUpdateUserClass(e)
            } else {
                Error::CouldNotUpdateUserClass(e)
            }
        })?;

        // Propagate changes to all users with this class (only if something changed)
        self.propagate_user_class_changes(
            &edited_class.name,
            &old_class.new_permissions,
            &edited_class.new_permissions,
            old_class.max_snatches_per_day,
            edited_class.max_snatches_per_day,
        )
        .await?;

        Ok(updated_class)
    }

    /// Propagates user class changes to all users with the given class.
    /// Permissions diff is applied to users in the edited class AND every class
    /// transitively above it in the promotion chain (descendants via
    /// `previous_user_class`), so higher classes inherit added permissions and
    /// drop removed ones. max_snatches_per_day stays scoped to the edited class
    /// since each class owns its own limit. Tracker is notified for the direct
    /// class users when max_snatches_per_day changed.
    async fn propagate_user_class_changes(
        &self,
        class_name: &str,
        old_permissions: &[UserPermission],
        new_permissions: &[UserPermission],
        old_max_snatches_per_day: Option<i32>,
        new_max_snatches_per_day: Option<i32>,
    ) -> Result<()> {
        let permissions_changed = old_permissions != new_permissions;
        let max_snatches_changed = old_max_snatches_per_day != new_max_snatches_per_day;

        // Skip if nothing changed
        if !permissions_changed && !max_snatches_changed {
            return Ok(());
        }

        if permissions_changed {
            // Formula: new_permissions = (current - old_class) + new_class
            // Applied to users in the edited class and every descendant class
            // (classes whose previous_user_class chain leads back here).
            sqlx::query!(
                r#"
                    WITH RECURSIVE descendants AS (
                        SELECT name FROM user_classes WHERE name = $1
                        UNION ALL
                        SELECT uc.name
                        FROM user_classes uc
                        JOIN descendants d ON uc.previous_user_class = d.name
                    )
                    UPDATE users
                    SET permissions = (
                        SELECT COALESCE(array_agg(p), ARRAY[]::user_permissions_enum[])
                        FROM (
                            SELECT unnest(permissions) AS p
                            EXCEPT
                            SELECT unnest($2::user_permissions_enum[])
                            UNION
                            SELECT unnest($3::user_permissions_enum[])
                        ) AS combined
                    )
                    WHERE class_name IN (SELECT name FROM descendants)
                "#,
                class_name,
                old_permissions as &[UserPermission],
                new_permissions as &[UserPermission],
            )
            .execute(&self.pool)
            .await?;
        }

        // Only notify tracker if max_snatches_per_day changed
        if !max_snatches_changed {
            return Ok(());
        }

        let affected_user_ids = sqlx::query_scalar!(
            r#"
                UPDATE users
                SET max_snatches_per_day = $2
                WHERE class_name = $1
                RETURNING id
            "#,
            class_name,
            new_max_snatches_per_day,
        )
        .fetch_all(&self.pool)
        .await?;

        for user_id in affected_user_ids {
            let payload = APIUpdateUserMaxSnatchesPerDay {
                id: user_id as u32,
                max_snatches_per_day: new_max_snatches_per_day.map(|x| x as u32),
            };

            self.notify_tracker(
                Method::PUT,
                &["users", &user_id.to_string(), "max-snatches-per-day"],
                Some(&payload),
                &format!("update user {user_id} snatch limit"),
            )
            .await;
        }

        Ok(())
    }

    pub async fn delete_user_class(&self, name: &str, target_class_name: &str) -> Result<()> {
        // Verify target class exists
        self.get_user_class_by_name(target_class_name).await?;

        // Migrate all users from the deleted class to the target class
        sqlx::query!(
            r#"
                UPDATE users
                SET class_name = $2
                WHERE class_name = $1
            "#,
            name,
            target_class_name
        )
        .execute(&self.pool)
        .await?;

        // Delete the user class
        let result = sqlx::query!(r#"DELETE FROM user_classes WHERE name = $1"#, name)
            .execute(&self.pool)
            .await
            .map_err(Error::CouldNotDeleteUserClass)?;

        if result.rows_affected() == 0 {
            return Err(Error::UserClassNotFound(name.to_string()));
        }

        Ok(())
    }

    pub async fn update_user_permissions(
        &self,
        user_id: i32,
        permissions: &[UserPermission],
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE users
                SET permissions = $2
                WHERE id = $1
            "#,
            user_id,
            permissions as &[UserPermission]
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn lock_user_class(&self, user_id: i32, locked: bool) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE users
                SET class_locked = $2
                WHERE id = $1
            "#,
            user_id,
            locked
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Changes the user's class and updates their permissions accordingly.
    /// Set `notify_tracker` to false in tests to skip the HTTP call to the tracker.
    pub async fn change_user_class(
        &self,
        user_id: i32,
        new_class_name: &str,
        notify_tracker: bool,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Get current user with their class and permissions
        let user = sqlx::query_as!(
            crate::models::user::User,
            r#"
                SELECT id, username, avatar, email, password_hash, registered_from_ip, created_at,
                       description, uploaded, real_uploaded, downloaded, real_downloaded, last_seen,
                       class_name, class_locked, permissions as "permissions: Vec<UserPermission>",
                       title_groups, edition_groups, torrents, forum_posts, forum_threads,
                       title_group_comments, request_comments, artist_comments, seeding, leeching,
                       snatched, seeding_size, requests_filled, collages_started, requests_voted,
                       average_seeding_time, invited, invitations, bonus_points, freeleech_tokens,
                       warned, banned, passkey, css_sheet_name, current_streak,
                       highest_streak, custom_title, max_snatches_per_day, irc_password, irc_site_embed_enabled, show_upload_method
                FROM users
                WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| Error::UserWithIdNotFound(user_id))?;

        let old_class_name = user.class_name.clone();

        // If the class hasn't changed, do nothing
        if old_class_name == new_class_name {
            return Ok(());
        }

        // Get old class details
        let old_class = sqlx::query_as!(
            crate::models::user::UserClass,
            r#"
                SELECT
                    name,
                    new_permissions as "new_permissions: Vec<UserPermission>",
                    max_snatches_per_day,
                    automatic_promotion,
                    automatic_demotion,
                    promotion_allowed_while_warned,
                    previous_user_class,
                    required_account_age_in_days,
                    required_ratio,
                    required_torrent_uploads,
                    required_torrent_uploads_in_unique_title_groups,
                    required_uploaded,
                    required_torrent_snatched,
                    required_downloaded,
                    required_forum_posts,
                    required_forum_posts_in_unique_threads,
                    required_title_group_comments,
                    required_seeding_size,
                    promotion_cost_bonus_points
                FROM user_classes
                WHERE name = $1
            "#,
            old_class_name
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| Error::UserClassNotFound(old_class_name.clone()))?;

        // Get new class details
        let new_class = sqlx::query_as!(
            crate::models::user::UserClass,
            r#"
                SELECT
                    name,
                    new_permissions as "new_permissions: Vec<UserPermission>",
                    automatic_promotion,
                    automatic_demotion,
                    promotion_allowed_while_warned,
                    previous_user_class,
                    max_snatches_per_day,
                    required_account_age_in_days,
                    required_ratio,
                    required_torrent_uploads,
                    required_torrent_uploads_in_unique_title_groups,
                    required_uploaded,
                    required_torrent_snatched,
                    required_downloaded,
                    required_forum_posts,
                    required_forum_posts_in_unique_threads,
                    required_title_group_comments,
                    required_seeding_size,
                    promotion_cost_bonus_points
                FROM user_classes
                WHERE name = $1
            "#,
            new_class_name
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| Error::UserClassNotFound(new_class_name.to_string()))?;

        let mut updated_permissions = user.permissions.clone();

        // Determine if it's a 1-hop promotion or demotion
        let is_promotion = new_class.previous_user_class.as_ref() == Some(&old_class_name);
        let is_demotion =
            old_class.previous_user_class.as_ref() == Some(&new_class_name.to_string());

        if is_promotion {
            // Add new class permissions (avoiding duplicates)
            for perm in &new_class.new_permissions {
                if !updated_permissions.contains(perm) {
                    updated_permissions.push(perm.clone());
                }
            }
        } else if is_demotion {
            // Remove old class permissions
            updated_permissions.retain(|p| !old_class.new_permissions.contains(p));
        }

        // Deduplicate permissions to ensure no duplicates exist
        let mut deduped = Vec::new();
        for perm in updated_permissions {
            if !deduped.contains(&perm) {
                deduped.push(perm);
            }
        }
        let updated_permissions = deduped;

        // Update user's class and permissions
        sqlx::query!(
            r#"
                UPDATE users
                SET class_name = $2, permissions = $3, max_snatches_per_day = $4
                WHERE id = $1
            "#,
            user_id,
            new_class_name,
            &updated_permissions as &[UserPermission],
            new_class.max_snatches_per_day
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Notify tracker about the new max_snatches_per_day
        if notify_tracker {
            let payload = APIUpdateUserMaxSnatchesPerDay {
                id: user_id as u32,
                max_snatches_per_day: new_class.max_snatches_per_day.map(|x| x as u32),
            };

            self.notify_tracker(
                Method::PUT,
                &["users", &user_id.to_string(), "max-snatches-per-day"],
                Some(&payload),
                &format!("update user {user_id} snatch limit"),
            )
            .await;
        }

        Ok(())
    }

    pub async fn find_users_lite(&self, username: &String) -> Result<Vec<UserLite>> {
        let found_users = sqlx::query_as!(
            UserLite,
            r#"
            SELECT id, username, warned, banned
            FROM users
            WHERE LOWER(username) LIKE LOWER('%' || $1 || '%')
            LIMIT 5
            "#,
            username
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotSearchForUsers)?;

        Ok(found_users)
    }

    pub async fn get_user_torrent_clients(
        &self,
        user_id: i32,
    ) -> Result<Vec<crate::models::peer::TorrentClient>> {
        let clients = sqlx::query_as!(
            crate::models::peer::TorrentClient,
            r#"
            SELECT
                ip,
                port,
                MIN(created_at)::timestamptz AS "first_seen_at!",
                MAX(updated_at)::timestamptz AS "last_seen_at!",
                SUM(uploaded)::bigint AS "real_uploaded!",
                SUM(downloaded)::bigint AS "real_downloaded!",
                agent
            FROM peers
            WHERE user_id = $1
            GROUP BY agent, ip, port
            ORDER BY agent, ip, port
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(clients)
    }

    pub async fn search_users(
        &self,
        query: &SearchUsersQuery,
    ) -> Result<PaginatedResults<UserSearchResult>> {
        let limit = query.page_size as i64;
        let offset = (query.page - 1) as i64 * query.page_size as i64;
        let order_by = query.order_by.to_string();
        let is_asc = matches!(
            query.order_by_direction,
            crate::models::common::OrderByDirection::Asc
        );
        let searched_permissions = query.permissions.as_deref().unwrap_or_default();
        let match_all_permissions =
            matches!(query.permissions_match, Some(UserPermissionMatchMode::All));

        let results = sqlx::query_as!(
            UserSearchResult,
            r#"
            SELECT id, username, avatar, class_name,
                   created_at AS "created_at?", last_seen AS "last_seen?",
                   uploaded AS "uploaded?", downloaded AS "downloaded?",
                   torrents AS "torrents?", title_groups AS "title_groups?",
                   title_group_comments AS "title_group_comments?",
                   forum_posts AS "forum_posts?", forum_threads AS "forum_threads?",
                   bonus_points AS "bonus_points?", seeding AS "seeding?", warned, banned,
                   paranoia_hidden_stats AS "paranoia_hidden_stats: Vec<DisplayableUserStats>"
            FROM users
            WHERE ($1::TEXT IS NULL OR LOWER(username) LIKE LOWER('%' || $1 || '%'))
              AND ($6::TIMESTAMPTZ IS NULL OR created_at >= $6)
              AND ($7::TIMESTAMPTZ IS NULL OR created_at <= $7)
              AND (COALESCE(cardinality($8::user_permissions_enum[]), 0) = 0
                   OR ($9 AND permissions @> $8)
                   OR (NOT $9 AND permissions && $8))
            ORDER BY
                CASE WHEN $2 = 'username' AND $3 THEN username END ASC,
                CASE WHEN $2 = 'username' AND NOT $3 THEN username END DESC,
                CASE WHEN $2 = 'created_at' AND $3 THEN created_at END ASC,
                CASE WHEN $2 = 'created_at' AND NOT $3 THEN created_at END DESC,
                CASE WHEN $2 = 'last_seen' AND $3 THEN last_seen END ASC,
                CASE WHEN $2 = 'last_seen' AND NOT $3 THEN last_seen END DESC,
                CASE WHEN $2 = 'uploaded' AND $3 THEN uploaded END ASC,
                CASE WHEN $2 = 'uploaded' AND NOT $3 THEN uploaded END DESC,
                CASE WHEN $2 = 'downloaded' AND $3 THEN downloaded END ASC,
                CASE WHEN $2 = 'downloaded' AND NOT $3 THEN downloaded END DESC,
                CASE WHEN $2 = 'torrents' AND $3 THEN torrents END ASC,
                CASE WHEN $2 = 'torrents' AND NOT $3 THEN torrents END DESC,
                CASE WHEN $2 = 'title_groups' AND $3 THEN title_groups END ASC,
                CASE WHEN $2 = 'title_groups' AND NOT $3 THEN title_groups END DESC,
                CASE WHEN $2 = 'title_group_comments' AND $3 THEN title_group_comments END ASC,
                CASE WHEN $2 = 'title_group_comments' AND NOT $3 THEN title_group_comments END DESC,
                CASE WHEN $2 = 'forum_posts' AND $3 THEN forum_posts END ASC,
                CASE WHEN $2 = 'forum_posts' AND NOT $3 THEN forum_posts END DESC,
                CASE WHEN $2 = 'forum_threads' AND $3 THEN forum_threads END ASC,
                CASE WHEN $2 = 'forum_threads' AND NOT $3 THEN forum_threads END DESC,
                CASE WHEN $2 = 'bonus_points' AND $3 THEN bonus_points END ASC,
                CASE WHEN $2 = 'bonus_points' AND NOT $3 THEN bonus_points END DESC,
                CASE WHEN $2 = 'seeding' AND $3 THEN seeding END ASC,
                CASE WHEN $2 = 'seeding' AND NOT $3 THEN seeding END DESC
            LIMIT $4 OFFSET $5
            "#,
            query.username,
            order_by,
            is_asc,
            limit,
            offset,
            query.registered_after,
            query.registered_before,
            searched_permissions as &[UserPermission],
            match_all_permissions
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotSearchForUsers)?;

        let total_items = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM users
            WHERE ($1::TEXT IS NULL OR LOWER(username) LIKE LOWER('%' || $1 || '%'))
              AND ($2::TIMESTAMPTZ IS NULL OR created_at >= $2)
              AND ($3::TIMESTAMPTZ IS NULL OR created_at <= $3)
              AND (COALESCE(cardinality($4::user_permissions_enum[]), 0) = 0
                   OR ($5 AND permissions @> $4)
                   OR (NOT $5 AND permissions && $4))
            "#,
            query.username,
            query.registered_after,
            query.registered_before,
            searched_permissions as &[UserPermission],
            match_all_permissions
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotSearchForUsers)?;

        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
        })
    }

    /// Returns every user (id and username) matching the given username/registration-date/
    /// permission filter, ignoring pagination. Used to contact all matching users at once.
    pub async fn find_users_matching_filter(
        &self,
        username: &Option<String>,
        registered_after: &Option<DateTime<Utc>>,
        registered_before: &Option<DateTime<Utc>>,
        searched_permissions: &[UserPermission],
        match_all_permissions: bool,
    ) -> Result<Vec<MassMessageRecipient>> {
        let recipients = sqlx::query_as!(
            MassMessageRecipient,
            r#"
            SELECT id, username
            FROM users
            WHERE ($1::TEXT IS NULL OR LOWER(username) LIKE LOWER('%' || $1 || '%'))
              AND ($2::TIMESTAMPTZ IS NULL OR created_at >= $2)
              AND ($3::TIMESTAMPTZ IS NULL OR created_at <= $3)
              AND (COALESCE(cardinality($4::user_permissions_enum[]), 0) = 0
                   OR ($5 AND permissions @> $4)
                   OR (NOT $5 AND permissions && $4))
            "#,
            username.as_deref(),
            *registered_after,
            *registered_before,
            searched_permissions as &[UserPermission],
            match_all_permissions
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotSearchForUsers)?;

        Ok(recipients)
    }

    pub async fn get_user_stats(&self, user_id: i32) -> Result<UserWithStats> {
        sqlx::query_as!(
            UserWithStats,
            r#"
            SELECT
                u.id,
                u.class_name,
                u.class_locked,
                u.warned,
                u.bonus_points,
                u.created_at,
                u.uploaded,
                u.downloaded,
                u.snatched,
                u.forum_posts,
                u.seeding_size,
                COALESCE(
                    (SELECT COUNT(*)
                     FROM torrents t
                     WHERE t.created_by_id = u.id),
                    0
                )::int as "torrent_uploads!",
                COALESCE(
                    (SELECT COUNT(DISTINCT eg.title_group_id)
                     FROM torrents t
                     INNER JOIN edition_groups eg ON t.edition_group_id = eg.id
                     WHERE t.created_by_id = u.id),
                    0
                )::int as "torrent_uploads_in_unique_title_groups!",
                COALESCE(
                    (SELECT COUNT(*)
                     FROM title_group_comments tgc
                     WHERE tgc.created_by_id = u.id),
                    0
                )::int as "title_group_comments!",
                COALESCE(
                    (SELECT COUNT(DISTINCT fp.forum_thread_id)
                     FROM forum_posts fp
                     WHERE fp.created_by_id = u.id),
                    0
                )::int as "forum_posts_in_unique_threads!"
            FROM users u
            WHERE u.id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::UserWithIdNotFound(user_id))
    }

    /// Get the users that can still change class (not banned, not class locked and belonging to
    /// one of the given classes), along with the statistics needed to evaluate the requirements.
    /// The pagination is done with a keyset on the user id, `last_user_id` being exclusive.
    pub async fn get_users_with_stats(
        &self,
        limit: i64,
        last_user_id: i32,
        class_names: &[String],
    ) -> Result<Vec<UserWithStats>> {
        sqlx::query_as!(
            UserWithStats,
            r#"
            SELECT
                u.id,
                u.class_name,
                u.class_locked,
                u.warned,
                u.bonus_points,
                u.created_at,
                u.uploaded,
                u.downloaded,
                u.snatched,
                u.forum_posts,
                u.seeding_size,
                -- maintained by the repository on upload and recomputed by the maintenance task
                u.torrents as "torrent_uploads!",
                COALESCE(
                    (SELECT COUNT(DISTINCT eg.title_group_id)
                     FROM torrents t
                     INNER JOIN edition_groups eg ON t.edition_group_id = eg.id
                     WHERE t.created_by_id = u.id),
                    0
                )::int as "torrent_uploads_in_unique_title_groups!",
                -- maintained by the repository on comment creation and deletion
                u.title_group_comments,
                COALESCE(
                    (SELECT COUNT(DISTINCT fp.forum_thread_id)
                     FROM forum_posts fp
                     WHERE fp.created_by_id = u.id),
                    0
                )::int as "forum_posts_in_unique_threads!"
            FROM users u
            WHERE u.banned = false
              AND u.class_locked = false
              AND u.class_name = ANY($3::varchar[])
              AND u.id > $2
            ORDER BY u.id
            LIMIT $1
            "#,
            limit,
            last_user_id,
            class_names
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::from)
    }

    /// Get the next class in the hierarchy for a user (the class that has previous_user_class = current class)
    pub async fn get_next_user_class(&self, current_class_name: &str) -> Result<Option<UserClass>> {
        sqlx::query_as!(
            UserClass,
            r#"
            SELECT
                name,
                new_permissions as "new_permissions: Vec<UserPermission>",
                max_snatches_per_day,
                automatic_promotion,
                automatic_demotion,
                promotion_allowed_while_warned,
                previous_user_class,
                required_account_age_in_days,
                required_ratio,
                required_torrent_uploads,
                required_torrent_uploads_in_unique_title_groups,
                required_uploaded,
                required_torrent_snatched,
                required_downloaded,
                required_forum_posts,
                required_forum_posts_in_unique_threads,
                required_title_group_comments,
                required_seeding_size,
                promotion_cost_bonus_points
            FROM user_classes
            WHERE previous_user_class = $1
            "#,
            current_class_name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::from)
    }

    /// Bans users who haven't been active for the given number of days.
    /// Skips users who are already banned or have the ImmuneActivityPruning permission.
    /// Creates a warning entry for each banned user.
    /// Returns the number of users banned.
    pub async fn ban_inactive_users(&self, inactive_days: i32) -> Result<u64> {
        let mut tx = self.pool.begin().await?;

        let banned_user_ids = sqlx::query_scalar!(
            r#"
            UPDATE users
            SET banned = true, warned = true
            WHERE last_seen < NOW() - make_interval(days => $1)
              AND banned = false
              AND NOT ('immune_activity_pruning' = ANY(permissions))
            RETURNING id
            "#,
            inactive_days
        )
        .fetch_all(&mut *tx)
        .await?;

        let banned_count = banned_user_ids.len() as u64;

        if !banned_user_ids.is_empty() {
            sqlx::query!(
                r#"
                INSERT INTO user_warnings (user_id, created_by_id, reason, expires_at, ban)
                SELECT unnest($1::int[]), 1, $2, NULL, true
                "#,
                &banned_user_ids,
                &format!("Inactivity: {} days", inactive_days)
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(banned_count)
    }

    /// Deduct bonus points from a user and promote them to the next class
    pub async fn purchase_promotion(
        &self,
        user_id: i32,
        new_class_name: &str,
        bonus_points_cost: i64,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Deduct bonus points
        sqlx::query!(
            r#"
            UPDATE users
            SET bonus_points = bonus_points - $2
            WHERE id = $1
            "#,
            user_id,
            bonus_points_cost
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        Self::log_bonus_points_change_tx(
            &mut tx,
            user_id,
            BonusPointsLogAction::ShopPurchasePromotion,
            -bonus_points_cost,
            Some(new_class_name),
            None,
        )
        .await?;

        tx.commit().await?;

        self.change_user_class(user_id, new_class_name, true).await
    }
}
