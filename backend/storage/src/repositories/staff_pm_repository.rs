use crate::{
    connection_pool::ConnectionPool,
    models::{
        notification::NotificationEvent,
        staff_pm::{StaffPm, StaffPmMessage, UserCreatedStaffPm, UserCreatedStaffPmMessage},
    },
};
use arcadia_common::error::{Error, Result};
use serde_json::Value;
use tokio::sync::broadcast;

impl ConnectionPool {
    pub async fn create_staff_pm(
        &self,
        conversation: &mut UserCreatedStaffPm,
        current_user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<StaffPm> {
        let created_conversation = sqlx::query_as!(
            StaffPm,
            r#"
				INSERT INTO staff_pms (subject, created_by_id)
				VALUES ($1, $2)
				RETURNING id, created_at, subject, created_by_id, resolved
			"#,
            conversation.subject,
            current_user_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateConversation)?;

        conversation.first_message.staff_pm_id = created_conversation.id;
        self.create_staff_pm_message(
            &conversation.first_message,
            current_user_id,
            true,
            notification_sender,
        )
        .await?;

        Ok(created_conversation)
    }

    pub async fn create_staff_pm_message(
        &self,
        message: &UserCreatedStaffPmMessage,
        current_user_id: i32,
        can_reply_staff_pm: bool,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<StaffPmMessage> {
        let mut tx = self.pool.begin().await?;

        // Check if the user created the staff PM and if it's resolved
        let staff_pm = sqlx::query!(
            r#"
                SELECT created_by_id, resolved
                FROM staff_pms
                WHERE id = $1
            "#,
            message.staff_pm_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotFindConversation)?;

        // If the staff PM is resolved, deny replies
        if staff_pm.resolved {
            return Err(Error::StaffPmResolved);
        }

        // If the user didn't create the staff PM and doesn't have permission to reply, deny access
        if staff_pm.created_by_id != current_user_id && !can_reply_staff_pm {
            return Err(Error::InsufficientPermissions(format!(
                "{:?}",
                crate::models::user::UserPermission::ReplyStaffPm
            )));
        }

        let created_message = sqlx::query_as!(
            StaffPmMessage,
            r#"
				INSERT INTO staff_pm_messages (staff_pm_id, created_by_id, content)
				VALUES ($1, $2, $3)
				RETURNING id, staff_pm_id, created_at, created_by_id, content
			"#,
            message.staff_pm_id,
            current_user_id,
            message.content,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateConversation)?;

        let user_ids = Self::notify_users_staff_pm_messages(
            &mut tx,
            message.staff_pm_id,
            created_message.id,
            current_user_id,
        )
        .await?;

        tx.commit().await?;

        if !user_ids.is_empty() {
            let _ = notification_sender.send(NotificationEvent::StaffPmMessage { user_ids });
        }

        Ok(created_message)
    }

    pub async fn resolve_staff_pm(
        &self,
        staff_pm_id: i64,
        current_user_id: i32,
        can_resolve_staff_pm: bool,
    ) -> Result<StaffPm> {
        let staff_pm_creator_id = sqlx::query_scalar!(
            r#"
                SELECT created_by_id
                FROM staff_pms
                WHERE id = $1
            "#,
            staff_pm_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindConversation)?;

        if staff_pm_creator_id != current_user_id && !can_resolve_staff_pm {
            return Err(Error::InsufficientPermissions(format!(
                "{:?}",
                crate::models::user::UserPermission::ResolveStaffPm
            )));
        }

        let updated = sqlx::query_as!(
            StaffPm,
            r#"
				UPDATE staff_pms
				SET resolved = TRUE
				WHERE id = $1
				RETURNING id, created_at, subject, created_by_id, resolved
			"#,
            staff_pm_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateConversation)?;

        Ok(updated)
    }

    pub async fn unresolve_staff_pm(
        &self,
        staff_pm_id: i64,
        current_user_id: i32,
        can_unresolve_staff_pm: bool,
    ) -> Result<StaffPm> {
        let staff_pm_creator_id = sqlx::query_scalar!(
            r#"
                SELECT created_by_id
                FROM staff_pms
                WHERE id = $1
            "#,
            staff_pm_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindConversation)?;

        if staff_pm_creator_id != current_user_id && !can_unresolve_staff_pm {
            return Err(Error::InsufficientPermissions(format!(
                "{:?}",
                crate::models::user::UserPermission::UnresolveStaffPm
            )));
        }

        let updated = sqlx::query_as!(
            StaffPm,
            r#"
				UPDATE staff_pms
				SET resolved = FALSE
				WHERE id = $1
				RETURNING id, created_at, subject, created_by_id, resolved
			"#,
            staff_pm_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateConversation)?;

        Ok(updated)
    }

    pub async fn list_staff_pms(
        &self,
        current_user_id: i32,
        can_read_staff_pm: bool,
    ) -> Result<Value> {
        let row = sqlx::query_unchecked!(
            r#"
			SELECT
				COALESCE(
					jsonb_agg(
						jsonb_build_object(
							'id', sp.id,
							'created_at', sp.created_at,
							'subject', sp.subject,
							'created_by', json_build_object(
								'id', u_creator.id,
								'username', u_creator.username,
								'class_name', u_creator.class_name,
								'banned', u_creator.banned,
								'avatar', u_creator.avatar,
								'warned', u_creator.warned,
								'custom_title', u_creator.custom_title
							),
							'resolved', sp.resolved,
							'last_message', jsonb_build_object(
								'created_at', lm.created_at,
								'created_by', jsonb_build_object(
									'id', u_lm.id,
									'username', u_lm.username,
									'class_name', u_lm.class_name,
									'banned', u_lm.banned,
									'avatar', u_lm.avatar,
									'warned', u_lm.warned,
									'custom_title', u_lm.custom_title
								)
							)
						)
						ORDER BY lm.created_at DESC
					),
					'[]'::jsonb
				) AS "staff_pms_json: serde_json::Value"
			FROM staff_pms sp
			JOIN LATERAL (
				SELECT created_at, created_by_id
				FROM staff_pm_messages
				WHERE staff_pm_id = sp.id
				ORDER BY created_at DESC
				LIMIT 1
			) lm ON TRUE
			JOIN users u_lm ON lm.created_by_id = u_lm.id
			INNER JOIN users u_creator ON sp.created_by_id = u_creator.id
			WHERE ($2)::bool OR sp.created_by_id = $1;
			"#,
            current_user_id,
            can_read_staff_pm,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindConversations)?;

        Ok(row.staff_pms_json.unwrap())
    }

    pub async fn get_staff_pm(
        &self,
        staff_pm_id: i64,
        current_user_id: i32,
        can_read_staff_pm: bool,
    ) -> Result<Value> {
        let row = sqlx::query_unchecked!(
            r#"
			SELECT
				json_build_object(
					'id', sp.id,
					'created_at', sp.created_at,
					'subject', sp.subject,
					'created_by', json_build_object(
						'id', u_creator.id,
						'username', u_creator.username,
						'class_name', u_creator.class_name,
						'banned', u_creator.banned,
						'avatar', u_creator.avatar,
						'warned', u_creator.warned,
						'custom_title', u_creator.custom_title
					),
					'resolved', sp.resolved,
					'messages', (
						SELECT json_agg(json_build_object(
							'id', m.id,
							'created_at', m.created_at,
							'created_by', json_build_object(
								'id', u_msg.id,
								'username', u_msg.username,
								'class_name', u_msg.class_name,
								'banned', u_msg.banned,
								'avatar', u_msg.avatar,
								'warned', u_msg.warned,
								'custom_title', u_msg.custom_title
							),
							'content', m.content
						) ORDER BY m.created_at ASC)
						FROM staff_pm_messages m
						INNER JOIN users u_msg ON m.created_by_id = u_msg.id
						WHERE m.staff_pm_id = sp.id
					)
				) AS "conversation: serde_json::Value"
			FROM staff_pms sp
			INNER JOIN users u_creator ON sp.created_by_id = u_creator.id
			WHERE sp.id = $1 AND (($3)::bool OR sp.created_by_id = $2);
			"#,
            staff_pm_id,
            current_user_id,
            can_read_staff_pm,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindConversation)?;

        Ok(row.conversation.unwrap())
    }
}
