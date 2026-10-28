use crate::{
    connection_pool::ConnectionPool,
    models::{
        bonus_points_log::BonusPointsLogAction,
        gift::{Gift, UserCreatedGift},
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::{Postgres, Transaction};

impl ConnectionPool {
    pub async fn create_gift(&self, gift: &UserCreatedGift, current_user_id: i32) -> Result<Gift> {
        if gift.bonus_points < 0 || gift.freeleech_tokens < 0 {
            return Err(Error::GiftAmountNegative);
        }

        let mut tx = self.pool.begin().await?;

        Self::decrement_bonus_points_and_freeleech_tokens(
            &mut tx,
            current_user_id,
            gift.bonus_points,
            gift.freeleech_tokens,
        )
        .await?;

        Self::increment_bonus_points_and_freeleech_tokens(
            &mut tx,
            gift.receiver_id,
            gift.bonus_points,
            gift.freeleech_tokens,
        )
        .await?;

        let inserted_gift = sqlx::query_as!(
            Gift,
            r#"
                INSERT INTO gifts (message, sender_id, receiver_id, bonus_points, freeleech_tokens)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, sent_at, message, sender_id, receiver_id, bonus_points, freeleech_tokens
            "#,
            gift.message,
            current_user_id,
            gift.receiver_id,
            gift.bonus_points,
            gift.freeleech_tokens
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateGift)?;

        if gift.bonus_points > 0 {
            let usernames = sqlx::query!(
                r#"
                    SELECT
                        (SELECT username FROM users WHERE id = $1) AS "sender_username!",
                        (SELECT username FROM users WHERE id = $2) AS "receiver_username!"
                "#,
                current_user_id,
                gift.receiver_id
            )
            .fetch_one(&mut *tx)
            .await?;

            let sent_details = format!("to {}", usernames.receiver_username);
            let received_details = format!("from {}", usernames.sender_username);
            let gift_item_id = Some(inserted_gift.id);

            Self::log_bonus_points_change_tx(
                &mut tx,
                current_user_id,
                BonusPointsLogAction::GiftSent,
                -gift.bonus_points,
                Some(&sent_details),
                gift_item_id,
            )
            .await?;
            Self::log_bonus_points_change_tx(
                &mut tx,
                gift.receiver_id,
                BonusPointsLogAction::GiftReceived,
                gift.bonus_points,
                Some(&received_details),
                gift_item_id,
            )
            .await?;
        }

        tx.commit().await?;

        Ok(inserted_gift)
    }

    pub async fn decrement_bonus_points_and_freeleech_tokens(
        tx: &mut Transaction<'_, Postgres>,
        user_id: i32,
        bonus_points: i64,
        freeleech_tokens: i32,
    ) -> Result<()> {
        // the balance is locked until the debit commits, so that concurrent
        // debits can not all be checked against the same balance
        let balance = sqlx::query!(
            r#"
              SELECT bonus_points, freeleech_tokens FROM users
              WHERE id = $1
              FOR UPDATE
            "#,
            user_id
        )
        .fetch_one(&mut **tx)
        .await?;
        if balance.bonus_points < bonus_points {
            return Err(Error::NotEnoughBonusPointsAvailable);
        }
        if balance.freeleech_tokens < freeleech_tokens {
            return Err(Error::NotEnoughFreeleechTokensAvailable);
        }

        sqlx::query!(
            r#"
              UPDATE users SET bonus_points = bonus_points - $1,
              freeleech_tokens = freeleech_tokens - $2
              WHERE id = $3
            "#,
            bonus_points,
            freeleech_tokens,
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn increment_bonus_points_and_freeleech_tokens(
        tx: &mut Transaction<'_, Postgres>,
        user_id: i32,
        bonus_points: i64,
        freeleech_tokens: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
              UPDATE users SET bonus_points = bonus_points + $1,
              freeleech_tokens = freeleech_tokens + $2
              WHERE id = $3
            "#,
            bonus_points,
            freeleech_tokens,
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
