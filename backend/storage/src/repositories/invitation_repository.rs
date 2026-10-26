use crate::{
    connection_pool::ConnectionPool,
    models::{
        common::PaginatedResults,
        invitation::{Invitation, InvitationHierarchy, SearchSentInvitationsQuery, SentInvitation},
        user::UserLiteAvatar,
    },
};
use arcadia_common::error::{Error, Result};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};
use sqlx::{Postgres, Transaction};

impl ConnectionPool {
    pub async fn create_invitation(
        &self,
        invitation: &SentInvitation,
        current_user_id: i32,
        invitation_expiration_days: i32,
    ) -> Result<Invitation> {
        // TODO: retry if invitation_key already exists
        let invitation_key: String = Alphanumeric.sample_string(&mut rng(), 50);

        let mut tx = self.pool.begin().await?;

        let _ = Self::decrement_invitations_available(&mut tx, current_user_id).await;

        // TODO: make sure no invitation/user exists for this email address
        let created_invitation = sqlx::query_as!(
            Invitation,
            r#"
                INSERT INTO invitations (message, inviter_notes, invitation_key, sender_id, receiver_email, expires_at, user_application_id)
                VALUES ($1, $2, $3, $4, $5, NOW() + make_interval(days => $6), $7)
                RETURNING id, created_at, expires_at, message, inviter_notes, invitation_key, sender_id, receiver_email, receiver_id, user_application_id
            "#,
            invitation.message,
            invitation.inviter_notes,
            invitation_key,
            current_user_id,
            invitation.receiver_email,
            invitation_expiration_days,
            invitation.user_application_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateInvitation)?;

        if invitation.user_application_id.is_some() {
            sqlx::query!(
                r#"
                UPDATE user_applications
                SET status = 'accepted'
                WHERE id = $1;
            "#,
                invitation.user_application_id
            )
            .execute(&mut *tx)
            .await
            .map_err(Error::CouldNotCreateInvitation)?;
        }

        tx.commit().await?;

        Ok(created_invitation)
    }

    pub async fn does_unexpired_invitation_exist(
        &self,
        invitation_key: &str,
    ) -> Result<Invitation> {
        let invitation = sqlx::query_as!(
            Invitation,
            r#"
              SELECT id, created_at, expires_at, message, inviter_notes, invitation_key, sender_id, receiver_email, receiver_id, user_application_id FROM invitations
              WHERE invitation_key = $1
              AND expires_at > NOW()
            "#,
            invitation_key
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::InvitationKeyInvalid)?;

        Ok(invitation)
    }

    pub async fn decrement_invitations_available(
        tx: &mut Transaction<'_, Postgres>,
        current_user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
              UPDATE users SET invitations = invitations - 1
              WHERE id = $1
            "#,
            current_user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn search_sent_invitations(
        &self,
        query: &SearchSentInvitationsQuery,
        current_user_id: i32,
        show_foreign_invitations: bool,
    ) -> Result<PaginatedResults<InvitationHierarchy>> {
        let offset = ((query.page - 1) * query.page_size) as i64;
        let limit = query.page_size as i64;

        let total_items = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM invitations i
            LEFT JOIN users u ON i.receiver_id = u.id
            WHERE (i.sender_id = $1 OR $3 = true)
              AND ($2::TEXT IS NULL OR u.username ILIKE '%' || $2 || '%')
            "#,
            current_user_id,
            query.receiver_username,
            show_foreign_invitations,
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT
                i.id,
                i.created_at,
                i.expires_at,
                i.message,
                i.inviter_notes,
                i.invitation_key,
                i.sender_id,
                i.receiver_email,
                i.user_application_id,
                u.id AS "receiver_user_id: Option<i32>",
                u.username AS "receiver_username: Option<String>",
                u.class_name AS "receiver_class_name: Option<String>",
                u.banned AS "receiver_banned: Option<bool>",
                u.avatar AS "receiver_avatar: Option<String>",
                u.warned AS "receiver_warned: Option<bool>",
                u.custom_title AS "receiver_custom_title: Option<String>",
                s.id AS "sender_user_id: Option<i32>",
                s.username AS "sender_username: Option<String>",
                s.class_name AS "sender_class_name: Option<String>",
                s.banned AS "sender_banned: Option<bool>",
                s.avatar AS "sender_avatar: Option<String>",
                s.warned AS "sender_warned: Option<bool>",
                s.custom_title AS "sender_custom_title: Option<String>"
            FROM invitations i
            LEFT JOIN users u ON i.receiver_id = u.id
            LEFT JOIN users s ON i.sender_id = s.id
            WHERE (i.sender_id = $1 OR $7 = true)
              AND ($2::TEXT IS NULL OR u.username ILIKE '%' || $2 || '%')
            ORDER BY
                CASE WHEN $5 = 'created_at' AND $6 = 'asc' THEN i.created_at END ASC,
                CASE WHEN $5 = 'created_at' AND $6 = 'desc' THEN i.created_at END DESC,
                CASE WHEN $5 = 'receiver_username' AND $6 = 'asc' THEN u.username END ASC,
                CASE WHEN $5 = 'receiver_username' AND $6 = 'desc' THEN u.username END DESC
            OFFSET $3 LIMIT $4
            "#,
            current_user_id,
            query.receiver_username,
            offset,
            limit,
            query.order_by_column.to_string(),
            query.order_by_direction.to_string(),
            show_foreign_invitations,
        )
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| {
                let receiver = match (
                    row.receiver_user_id,
                    row.receiver_username,
                    row.receiver_class_name,
                    row.receiver_banned,
                    row.receiver_warned,
                ) {
                    (Some(id), Some(username), Some(class_name), Some(banned), Some(warned)) => {
                        Some(UserLiteAvatar {
                            id,
                            username,
                            class_name,
                            banned,
                            avatar: row.receiver_avatar.flatten(),
                            warned,
                            custom_title: row.receiver_custom_title.flatten(),
                        })
                    }
                    _ => None,
                };

                let sender = if show_foreign_invitations {
                    match (
                        row.sender_user_id,
                        row.sender_username,
                        row.sender_class_name,
                        row.sender_banned,
                        row.sender_warned,
                    ) {
                        (
                            Some(id),
                            Some(username),
                            Some(class_name),
                            Some(banned),
                            Some(warned),
                        ) => Some(UserLiteAvatar {
                            id,
                            username,
                            class_name,
                            banned,
                            avatar: row.sender_avatar.flatten(),
                            warned,
                            custom_title: row.sender_custom_title.flatten(),
                        }),
                        _ => None,
                    }
                } else {
                    None
                };

                InvitationHierarchy {
                    id: row.id,
                    created_at: row.created_at.into(),
                    expires_at: row.expires_at.into(),
                    message: row.message,
                    inviter_notes: row.inviter_notes,
                    invitation_key: row.invitation_key,
                    sender_id: row.sender_id,
                    sender,
                    receiver_email: row.receiver_email,
                    receiver,
                    user_application_id: row.user_application_id,
                }
            })
            .collect();

        Ok(PaginatedResults {
            results,
            total_items,
            page: query.page,
            page_size: query.page_size,
        })
    }
}
