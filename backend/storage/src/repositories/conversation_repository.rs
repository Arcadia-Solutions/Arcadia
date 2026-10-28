use crate::{
    connection_pool::ConnectionPool,
    models::{
        common::PaginatedResults,
        conversation::{
            Conversation, ConversationMessage, ConversationSearchQuery, ConversationSearchResult,
            MassMessageRequest, MassMessageResult, UserCreatedConversation,
            UserCreatedConversationMessage,
        },
        notification::NotificationEvent,
        user::UserPermissionMatchMode,
    },
};
use arcadia_common::error::{Error, Result};
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use tokio::sync::broadcast;

impl ConnectionPool {
    pub async fn create_conversation(
        &self,
        conversation: &mut UserCreatedConversation,
        current_user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<Conversation> {
        //TODO: make transactional
        let created_conversation = sqlx::query_as!(
            Conversation,
            r#"
                INSERT INTO conversations (subject, sender_id, receiver_id)
                VALUES ($1, $2, $3)
                RETURNING id, created_at, subject, sender_id, receiver_id, sender_last_seen_at, receiver_last_seen_at, locked
            "#,
            conversation.subject,
            current_user_id,
            conversation.receiver_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateConversation)?;

        conversation.first_message.conversation_id = created_conversation.id;
        self.create_conversation_message(
            &conversation.first_message,
            current_user_id,
            notification_sender,
        )
        .await?;

        Ok(created_conversation)
    }

    /// Sends a private message (a conversation with a single message) to every user
    /// matching the filter, skipping the sender. The `{username}` placeholder in the
    /// message is replaced per recipient with a link to their profile, the same way the
    /// automated signup message works. Returns the amount of messages sent.
    pub async fn create_mass_conversation(
        &self,
        payload: &MassMessageRequest,
        current_user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<MassMessageResult> {
        let recipients = self
            .find_users_matching_filter(
                &payload.username,
                &payload.registered_after,
                &payload.registered_before,
                payload.permissions.as_deref().unwrap_or_default(),
                matches!(
                    payload.permissions_match,
                    Some(UserPermissionMatchMode::All)
                ),
            )
            .await?;

        let mut messages_sent = 0;
        for recipient in recipients {
            if recipient.id == current_user_id {
                continue;
            }

            let content = payload.message.replace("{username}", &recipient.username);

            let mut conversation = UserCreatedConversation {
                subject: payload.subject.clone(),
                receiver_id: recipient.id,
                first_message: UserCreatedConversationMessage {
                    conversation_id: 0,
                    content,
                },
            };
            self.create_conversation(&mut conversation, current_user_id, notification_sender)
                .await?;
            messages_sent += 1;
        }

        Ok(MassMessageResult { messages_sent })
    }

    pub async fn create_conversation_message(
        &self,
        message: &UserCreatedConversationMessage,
        current_user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<ConversationMessage> {
        let result = sqlx::query_as!(
            ConversationMessage,
            r#"
                INSERT INTO conversation_messages (conversation_id, created_by_id, content)
                SELECT $1, $2, $3
                FROM conversations
                WHERE id = $1 AND NOT locked AND $2 IN (sender_id, receiver_id)
                RETURNING id, conversation_id, created_at, created_by_id, content
            "#,
            message.conversation_id,
            current_user_id,
            message.content,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::CouldNotCreateConversation)?;

        match result {
            Some(msg) => {
                let notification_info = sqlx::query!(
                    r#"
                    SELECT
                        CASE WHEN c.sender_id = $2 THEN c.receiver_id ELSE c.sender_id END as "other_user_id!",
                        CASE
                            WHEN prev_msg.created_by_id IS NULL THEN false
                            WHEN prev_msg.created_by_id != $2 THEN false
                            WHEN c.sender_id = $2 THEN
                                c.receiver_last_seen_at IS NULL OR c.receiver_last_seen_at < prev_msg.created_at
                            ELSE
                                c.sender_last_seen_at < prev_msg.created_at
                        END as "was_already_unread!"
                    FROM conversations c
                    LEFT JOIN LATERAL (
                        SELECT cm.created_at, cm.created_by_id
                        FROM conversation_messages cm
                        WHERE cm.conversation_id = c.id AND cm.id != $3
                        ORDER BY cm.created_at DESC
                        LIMIT 1
                    ) AS prev_msg ON TRUE
                    WHERE c.id = $1
                    "#,
                    message.conversation_id,
                    current_user_id,
                    msg.id
                )
                .fetch_one(&self.pool)
                .await
                .map_err(Error::CouldNotFindConversation)?;

                if !notification_info.was_already_unread {
                    let _ = notification_sender.send(NotificationEvent::Conversation {
                        user_ids: vec![notification_info.other_user_id],
                    });
                }

                Ok(msg)
            }
            None => {
                // differentiate between not found and locked, a conversation the user is not
                // part of is reported as not found
                let is_locked = sqlx::query_scalar!(
                    r#"SELECT locked FROM conversations WHERE id = $1 AND $2 IN (sender_id, receiver_id)"#,
                    message.conversation_id,
                    current_user_id,
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(Error::CouldNotFindConversation)?;

                match is_locked {
                    Some(true) => Err(Error::ConversationLocked),
                    Some(false) => Err(Error::CouldNotCreateConversationMessage(
                        sqlx::Error::RowNotFound,
                    )),
                    None => Err(Error::CouldNotFindConversation(sqlx::Error::RowNotFound)),
                }
            }
        }
    }

    pub async fn search_conversations(
        &self,
        user_id: i32,
        query: &ConversationSearchQuery,
        bypass_membership_check: bool,
    ) -> Result<PaginatedResults<ConversationSearchResult>> {
        let limit = query.page_size as i64;
        let offset = (query.page - 1) as i64 * query.page_size as i64;
        let search_term = query.search_term.as_deref().filter(|s| !s.is_empty());
        let order_by_column = query.order_by_column.to_string();
        let order_by_direction = query.order_by_direction.to_string();

        let results = sqlx::query_as!(
            ConversationSearchResult,
            r#"
            SELECT
                c.id AS conversation_id,
                c.created_at AS conversation_created_at,
                c.subject,
                c.sender_id,
                c.receiver_id,
                c.sender_last_seen_at,
                c.receiver_last_seen_at,
                c.locked,
                co.id AS correspondant_id,
                co.username AS correspondant_username,
                co.warned AS correspondant_warned,
                co.banned AS correspondant_banned,
                s.username AS sender_username,
                s.warned AS sender_warned,
                s.banned AS sender_banned,
                r.username AS receiver_username,
                r.warned AS receiver_warned,
                r.banned AS receiver_banned,
                lm.messages_amount AS "messages_amount!",
                lm.created_at AS last_message_created_at,
                lm_user.id AS last_message_created_by_id,
                lm_user.username AS last_message_created_by_username
            FROM conversations AS c
            JOIN users AS s ON c.sender_id = s.id
            JOIN users AS r ON c.receiver_id = r.id
            JOIN LATERAL (
                SELECT cm.created_at, cm.created_by_id, COUNT(*) OVER () AS messages_amount
                FROM conversation_messages AS cm
                WHERE cm.conversation_id = c.id
                ORDER BY cm.created_at DESC
                LIMIT 1
            ) AS lm ON TRUE
            JOIN users AS lm_user ON lm.created_by_id = lm_user.id
            JOIN users AS co ON (CASE WHEN c.sender_id = $1 THEN c.receiver_id ELSE c.sender_id END) = co.id
            WHERE
                ($6 OR c.sender_id = $1 OR c.receiver_id = $1)
                AND ($7::INT IS NULL OR c.sender_id = $7 OR c.receiver_id = $7)
                AND (
                    $4::TEXT IS NULL
                    OR c.subject ILIKE '%' || $4 || '%'
                    OR co.username ILIKE '%' || $4 || '%'
                    OR (NOT $5 AND EXISTS (
                        SELECT 1 FROM conversation_messages cm
                        WHERE cm.conversation_id = c.id
                        AND cm.content ILIKE '%' || $4 || '%'
                    ))
                )
            ORDER BY
                CASE WHEN $8 = 'last_message' AND $9 = 'asc' THEN lm.created_at END ASC,
                CASE WHEN $8 = 'last_message' AND $9 = 'desc' THEN lm.created_at END DESC,
                CASE WHEN $8 = 'created_at' AND $9 = 'asc' THEN c.created_at END ASC,
                CASE WHEN $8 = 'created_at' AND $9 = 'desc' THEN c.created_at END DESC,
                CASE WHEN $8 = 'messages_amount' AND $9 = 'asc' THEN lm.messages_amount END ASC,
                CASE WHEN $8 = 'messages_amount' AND $9 = 'desc' THEN lm.messages_amount END DESC,
                CASE WHEN $8 = 'subject' AND $9 = 'asc' THEN c.subject END ASC,
                CASE WHEN $8 = 'subject' AND $9 = 'desc' THEN c.subject END DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset,
            search_term,
            query.search_titles_only,
            bypass_membership_check,
            query.user_id,
            order_by_column,
            order_by_direction,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotSearchConversations)?;

        let total_results = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM conversations AS c
            JOIN users AS co ON (CASE WHEN c.sender_id = $1 THEN c.receiver_id ELSE c.sender_id END) = co.id
            WHERE
                ($4 OR c.sender_id = $1 OR c.receiver_id = $1)
                AND ($5::INT IS NULL OR c.sender_id = $5 OR c.receiver_id = $5)
                AND (
                    $2::TEXT IS NULL
                    OR c.subject ILIKE '%' || $2 || '%'
                    OR co.username ILIKE '%' || $2 || '%'
                    OR (NOT $3 AND EXISTS (
                        SELECT 1 FROM conversation_messages cm
                        WHERE cm.conversation_id = c.id
                        AND cm.content ILIKE '%' || $2 || '%'
                    ))
                )
            "#,
            user_id,
            search_term,
            query.search_titles_only,
            bypass_membership_check,
            query.user_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotSearchConversations)?
        .unwrap_or(0);

        Ok(PaginatedResults {
            results,
            total_items: total_results,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// Will only update the read status if the user is part of the conversation
    /// So if the user is not part of it and sees it with UserPermission::ReadAllConversations
    /// the read status will be left unchanged for everyone.
    pub async fn find_conversation(
        &self,
        conversation_id: i64,
        current_user_id: i32,
        bypass_membership_check: bool,
    ) -> Result<Value> {
        let conversation_with_messages = sqlx::query!(
            r#"
            SELECT
                json_build_object(
                    'id', c.id,
                    'created_at', c.created_at,
                    'subject', c.subject,
                    'sender_last_seen_at', c.sender_last_seen_at,
                    'receiver_last_seen_at', c.receiver_last_seen_at,
                    'locked', c.locked,
                    'sender', json_build_object(
                        'id', s.id,
                        'username', s.username,
                        'class_name', s.class_name,
                        'custom_title', s.custom_title,
                        'banned', s.banned,
                        'avatar', s.avatar,
                        'warned', s.warned
                    ),
                    'receiver', json_build_object(
                        'id', r.id,
                        'username', r.username,
                        'class_name', r.class_name,
                        'custom_title', r.custom_title,
                        'banned', r.banned,
                        'avatar', r.avatar,
                        'warned', r.warned
                    ),
                    'messages', json_agg(json_build_object(
                        'id', m.id,
                        'created_at', m.created_at,
                        'content', m.content,
                        'created_by', json_build_object(
                            'id', u_msg.id,
                            'username', u_msg.username,
                            'class_name', u_msg.class_name,
                            'custom_title', u_msg.custom_title,
                            'banned', u_msg.banned,
                            'avatar', u_msg.avatar,
                            'warned', u_msg.warned
                        )
                    ) ORDER BY m.created_at ASC)
                ) AS conversation_details
            FROM
                conversations c
            INNER JOIN
                users s ON c.sender_id = s.id
            INNER JOIN
                users r ON c.receiver_id = r.id
            INNER JOIN
                conversation_messages m ON c.id = m.conversation_id
            INNER JOIN
                users u_msg ON m.created_by_id = u_msg.id
            WHERE
                c.id = $1 AND ($3 OR c.sender_id = $2 OR c.receiver_id = $2) -- prevent users from reading a conversation they're not part of
            GROUP BY
                c.id, c.created_at, c.subject, c.locked,
                s.id, s.username, s.class_name, s.custom_title, s.banned, s.avatar, s.warned,
                r.id, r.username, r.class_name, r.custom_title, r.banned, r.avatar, r.warned;
            "#,
            conversation_id,
            current_user_id,
            bypass_membership_check
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindConversation)?;

        sqlx::query!(
            r#"
            UPDATE conversations
            SET
                sender_last_seen_at = CASE
                    WHEN sender_id = $2 THEN NOW()
                    ELSE sender_last_seen_at
                END,
                receiver_last_seen_at = CASE
                    WHEN receiver_id = $2 THEN NOW()
                    ELSE receiver_last_seen_at
                END
            WHERE
                id = $1;
            "#,
            conversation_id,
            current_user_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(conversation_with_messages.conversation_details.unwrap())
    }

    /// Sends a message from a sender to multiple recipients, creating a new conversation for each.
    pub async fn send_batch_messages_tx(
        tx: &mut Transaction<'_, Postgres>,
        sender_id: i32,
        recipient_ids: &[i32],
        subject: &str,
        content: &str,
        locked: bool,
    ) -> Result<()> {
        for &recipient_id in recipient_ids {
            let conversation = sqlx::query_scalar!(
                r#"
                INSERT INTO conversations (subject, sender_id, receiver_id, locked)
                VALUES ($1, $2, $3, $4)
                RETURNING id
                "#,
                subject,
                sender_id,
                recipient_id,
                locked
            )
            .fetch_one(&mut **tx)
            .await
            .map_err(Error::CouldNotCreateConversation)?;

            sqlx::query!(
                r#"
                INSERT INTO conversation_messages (conversation_id, created_by_id, content)
                VALUES ($1, $2, $3)
                "#,
                conversation,
                sender_id,
                content
            )
            .execute(&mut **tx)
            .await
            .map_err(Error::CouldNotCreateConversation)?;
        }

        Ok(())
    }

    /// Sends a message from a sender to multiple recipients, in a transaction of its own.
    /// Callers that already have a transaction of their own should use
    /// [`Self::send_batch_messages_tx`] instead.
    pub async fn send_batch_messages(
        &self,
        sender_id: i32,
        recipient_ids: &[i32],
        subject: &str,
        content: &str,
        locked: bool,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        Self::send_batch_messages_tx(&mut tx, sender_id, recipient_ids, subject, content, locked)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
