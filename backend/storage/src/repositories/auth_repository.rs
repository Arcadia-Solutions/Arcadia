use crate::{
    connection_pool::ConnectionPool,
    models::{
        arcadia_settings::ArcadiaSettings,
        common::PaginatedResults,
        invitation::Invitation,
        unauthorized_access::{SearchUnauthorizedAccessQuery, UnauthorizedAccess},
        user::{
            APIKey, APIKeyScope, CreatedAPIKey, Login, Register, User, UserCreatedAPIKey,
            UserLiteAvatar, UserPermission,
        },
    },
};
use arcadia_common::error::{Error, Result};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use chrono::{DateTime, Duration, Utc};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng, RngExt,
};
use sha2::{Digest, Sha256};
use sqlx::types::ipnetwork::IpNetwork;
use std::sync::LazyLock;

pub const MAXIMUM_API_KEYS_PER_USER: i64 = 15;

const API_KEY_LENGTH: usize = 40;

/// API keys are never stored as is, only their sha256 hash is.
fn hash_api_key(api_key: &str) -> [u8; 32] {
    Sha256::digest(api_key.as_bytes()).into()
}

pub static PASSWORD_RESET_TOKEN_DURATION: LazyLock<Duration> = LazyLock::new(|| Duration::days(1));

const PASSWORD_RESET_TOKEN_LENGTH: usize = 50;

impl ConnectionPool {
    pub async fn does_username_exist(&self, username: &str) -> Result<bool> {
        let result = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)",
            username
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.exists.unwrap_or(false))
    }

    pub async fn create_user(
        &self,
        user: &Register,
        from_ip: IpNetwork,
        password_hash: &str,
        invitation: &Option<Invitation>,
        arcadia_settings: &ArcadiaSettings,
    ) -> Result<User> {
        let rng = rand::rng();

        // TODO: check if the passkey already exists
        let passkey: String = rng
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Check username availability first
        if self.does_username_exist(&user.username).await? {
            return Err(Error::UsernameAlreadyExists);
        }

        let mut tx = self.pool.begin().await?;

        let registered_user = sqlx::query_as!(
            User,
            r#"
                INSERT INTO users (username, email, password_hash, registered_from_ip, passkey, class_name, css_sheet_name, permissions, max_snatches_per_day, uploaded, downloaded, bonus_points, freeleech_tokens)
                SELECT $1, $2, $3, $4, $5, $6, $7, uc.new_permissions, uc.max_snatches_per_day, $8, $9, $10, $11
                FROM user_classes uc
                WHERE uc.name = $6::VARCHAR(30)
                RETURNING id, username, avatar, email, password_hash, registered_from_ip, created_at,
                          description, uploaded, real_uploaded, downloaded, real_downloaded, last_seen,
                          class_name, class_locked, permissions as "permissions: Vec<UserPermission>",
                          title_groups, edition_groups, torrents, forum_posts, forum_threads,
                          title_group_comments, request_comments, artist_comments, seeding, leeching,
                          snatched, seeding_size, requests_filled, collages_started, requests_voted,
                          average_seeding_time, invited, invitations, bonus_points, freeleech_tokens,
                          warned, banned, passkey, css_sheet_name, current_streak,
                          highest_streak, custom_title, max_snatches_per_day,
                          irc_password, irc_site_embed_enabled, show_upload_method
            "#,
            &user.username,
            &user.email,
            password_hash,
            from_ip,
            passkey,
            arcadia_settings.user_class_name_on_signup,
            arcadia_settings.default_css_sheet_name,
            arcadia_settings.default_user_uploaded_on_registration,
            arcadia_settings.default_user_downloaded_on_registration,
            arcadia_settings.default_user_bonus_points_on_registration,
            arcadia_settings.default_user_freeleech_tokens_on_registration
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateUser)?;

        if let Some(inv) = invitation {
            // the invitation is only consumed when it is still unused, so that two
            // registrations racing on the same key can not both claim it
            let consumed = sqlx::query!(
                r#"
                UPDATE invitations SET receiver_id = $1 WHERE id = $2 AND receiver_id IS NULL;
                "#,
                registered_user.id,
                inv.id
            )
            .execute(&mut *tx)
            .await?;

            if consumed.rows_affected() != 1 {
                // dropping the transaction rolls the created user back
                return Err(Error::InvitationKeyAlreadyUsed);
            }

            sqlx::query!(
                r#"
                UPDATE users SET invited = invited + 1 WHERE id = $1;
                "#,
                inv.sender_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(registered_user)
    }

    pub async fn find_user_with_password(&self, login: &Login) -> Result<User> {
        let user = sqlx::query_as!(
            User,
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
                WHERE username = $1
            "#,
            login.username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            log::debug!("Error fetching user: {:?}", e);
            Error::WrongUsernameOrPassword
        })?;

        let parsed_hash = PasswordHash::new(&user.password_hash);

        Argon2::default()
            .verify_password(login.password.as_bytes(), &parsed_hash.unwrap())
            .map_err(|_| Error::WrongUsernameOrPassword)?;

        Ok(user)
    }

    /// Finds the user owning the given API key and the scopes granted to that key, while
    /// marking the key as used.
    ///
    /// Marking the key only happens once a minute, as every request authenticated with an API
    /// key goes through here: writing on each of them would make the requests of a single key
    /// serialize on the row lock of that key.
    pub async fn find_user_id_and_scopes_with_api_key(
        &self,
        api_key: &str,
    ) -> Result<(i32, Vec<APIKeyScope>)> {
        let value_hash = hash_api_key(api_key);

        let api_key = sqlx::query!(
            r#"
            WITH usable_api_key AS (
                SELECT api_keys.id, api_keys.user_id, api_keys.scopes, api_keys.last_used_at
                FROM api_keys
                JOIN users ON users.id = api_keys.user_id
                WHERE api_keys.value_hash = $1 AND users.banned = FALSE
            ), mark_as_used AS (
                UPDATE api_keys SET last_used_at = NOW()
                WHERE id = (
                    SELECT id FROM usable_api_key
                    WHERE last_used_at IS NULL OR last_used_at < NOW() - INTERVAL '1 minute'
                )
            )
            SELECT user_id as "user_id!", scopes as "scopes!: Vec<APIKeyScope>"
            FROM usable_api_key
            "#,
            &value_hash[..]
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::InvalidAPIKeyOrBanned)?;

        Ok((api_key.user_id, api_key.scopes))
    }

    pub async fn find_api_keys(&self, current_user_id: i32) -> Result<Vec<APIKey>> {
        let api_keys = sqlx::query_as!(
            APIKey,
            r#"
            SELECT id, created_at, last_used_at, name, last_four,
                   scopes as "scopes: Vec<APIKeyScope>"
            FROM api_keys
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            current_user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(api_keys)
    }

    pub async fn delete_api_key(&self, api_key_id: i64, current_user_id: i32) -> Result<()> {
        let deleted_api_keys = sqlx::query!(
            r#"DELETE FROM api_keys WHERE id = $1 AND user_id = $2"#,
            api_key_id,
            current_user_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if deleted_api_keys == 0 {
            return Err(Error::APIKeyNotFound);
        }

        Ok(())
    }

    pub async fn find_user_with_id(&self, id: i32) -> Result<User> {
        sqlx::query_as!(
            User,
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
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::WrongUsernameOrPassword)
    }

    pub async fn create_api_key(
        &self,
        created_api_key: &UserCreatedAPIKey,
        current_user_id: i32,
    ) -> Result<CreatedAPIKey> {
        let value: String = Alphanumeric.sample_string(&mut rng(), API_KEY_LENGTH);
        let value_hash = hash_api_key(&value);

        // the insertion only happens while the user is below the maximum, so no row is
        // returned once they reached it
        let api_key = sqlx::query_as!(
            APIKey,
            r#"
            INSERT INTO api_keys (name, value_hash, last_four, scopes, user_id)
            SELECT $1::VARCHAR, $2::BYTEA, $3::VARCHAR, $4::api_key_scope_enum[], $5::INT
            WHERE (SELECT COUNT(*) FROM api_keys WHERE user_id = $5) < $6
            RETURNING id, created_at, last_used_at, name, last_four,
                      scopes as "scopes: Vec<APIKeyScope>"
            "#,
            created_api_key.name,
            &value_hash[..],
            &value[value.len() - 4..],
            &created_api_key.scopes as &[APIKeyScope],
            current_user_id,
            MAXIMUM_API_KEYS_PER_USER
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::CouldNotCreateAPIKey)?
        .ok_or(Error::TooManyAPIKeys(MAXIMUM_API_KEYS_PER_USER))?;

        Ok(CreatedAPIKey { api_key, value })
    }

    pub async fn user_has_permission(
        &self,
        user_id: i32,
        permission: &UserPermission,
    ) -> Result<bool> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users WHERE id = $1 AND $2 = ANY(permissions)
            ) as "exists!"
            "#,
            user_id,
            permission as &UserPermission
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn require_permission(
        &self,
        user_id: i32,
        permission: &UserPermission,
        path: &str,
    ) -> Result<()> {
        let has_permission = self.user_has_permission(user_id, permission).await?;

        if !has_permission {
            // Log unauthorized access
            let _ = sqlx::query!(
                r#"
                INSERT INTO unauthorized_accesses (user_id, missing_permission, path)
                VALUES ($1, $2, $3)
                "#,
                user_id,
                permission as &UserPermission,
                path
            )
            .execute(&self.pool)
            .await;

            return Err(Error::InsufficientPermissions(format!("{:?}", permission)));
        }

        Ok(())
    }

    pub async fn find_unauthorized_accesses(
        &self,
        query: SearchUnauthorizedAccessQuery,
    ) -> Result<PaginatedResults<UnauthorizedAccess>> {
        let total_items: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM unauthorized_accesses
            WHERE ($1::INT IS NULL OR user_id = $1)
              AND created_at >= $2
              AND created_at <= $3
              AND ($4::user_permissions_enum IS NULL OR missing_permission = $4)
            "#,
            query.user_id,
            query.from_date,
            query.to_date,
            query.permission.clone() as Option<UserPermission>
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT
                ua.id,
                ua.created_at,
                u.id as user_id,
                u.username,
                u.class_name,
                u.banned,
                u.avatar,
                u.warned,
                u.custom_title,
                ua.missing_permission as "missing_permission: UserPermission",
                ua.path
            FROM unauthorized_accesses ua
            JOIN users u ON ua.user_id = u.id
            WHERE ($1::INT IS NULL OR ua.user_id = $1)
              AND ua.created_at >= $2
              AND ua.created_at <= $3
              AND ($4::user_permissions_enum IS NULL OR ua.missing_permission = $4)
            ORDER BY
              CASE WHEN $5 = 'missing_permission' AND $6 = 'asc' THEN ua.missing_permission END ASC,
              CASE WHEN $5 = 'missing_permission' AND $6 = 'desc' THEN ua.missing_permission END DESC,
              CASE WHEN $5 = 'created_at' AND $6 = 'asc' THEN ua.created_at END ASC,
              CASE WHEN $5 = 'created_at' AND $6 = 'desc' THEN ua.created_at END DESC
            OFFSET ($7 - 1) * LEAST($8, 100)
            LIMIT LEAST($8, 100)
            "#,
            query.user_id,
            query.from_date,
            query.to_date,
            query.permission as Option<UserPermission>,
            query.sort_by_column.to_string(),
            query.sort_by_direction.to_string(),
            query.page as i32,
            query.page_size as i32
        )
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| UnauthorizedAccess {
                id: row.id,
                created_at: row.created_at,
                user: UserLiteAvatar {
                    id: row.user_id,
                    username: row.username,
                    class_name: row.class_name,
                    banned: row.banned,
                    avatar: row.avatar,
                    warned: row.warned,
                    custom_title: row.custom_title,
                },
                missing_permission: row.missing_permission,
                path: row.path,
            })
            .collect();

        Ok(PaginatedResults {
            results,
            total_items,
            page: query.page as u32,
            page_size: query.page_size.min(100) as u32,
        })
    }

    pub async fn update_user_password_hash(&self, user_id: i32, password_hash: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users SET password_hash = $2 WHERE id = $1
            "#,
            user_id,
            password_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Creates a single use token allowing an unauthenticated user to set a new password,
    /// and returns it along with the moment it expires. A user only ever has one valid token,
    /// so creating a new one revokes the previous one.
    pub async fn create_password_reset_token(
        &self,
        user_id: i32,
    ) -> Result<(String, DateTime<Utc>)> {
        let token = Alphanumeric.sample_string(&mut rng(), PASSWORD_RESET_TOKEN_LENGTH);
        let expires_at = Utc::now() + *PASSWORD_RESET_TOKEN_DURATION;

        sqlx::query!(
            r#"
            INSERT INTO password_reset_tokens (value, user_id, expires_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id) DO UPDATE
            SET value = EXCLUDED.value, expires_at = EXCLUDED.expires_at, created_at = NOW()
            "#,
            token,
            user_id,
            expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok((token, expires_at))
    }

    /// Revokes the password reset token of the user, if they have one. Called whenever their
    /// password changes, so that a link that has been used, or that has been superseded by a
    /// password change, can not be used anymore.
    pub async fn revoke_password_reset_token(&self, user_id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM password_reset_tokens WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Returns the id of the user the password reset token belongs to, without consuming it.
    pub async fn find_password_reset_token_user(&self, token: &str) -> Result<i32> {
        sqlx::query_scalar!(
            r#"
            SELECT user_id FROM password_reset_tokens WHERE value = $1 AND expires_at > NOW()
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(Error::InvalidOrExpiredPasswordResetToken)
    }

    pub async fn set_irc_password(&self, user_id: i32, password: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users SET irc_password = $2 WHERE id = $1
            "#,
            user_id,
            password
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_user_by_username(&self, username: &str) -> Result<User> {
        sqlx::query_as!(
            User,
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
                WHERE username = $1
            "#,
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::UserNotFound(username.to_string()))
    }
}
