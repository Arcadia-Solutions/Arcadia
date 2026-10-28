use crate::{connection_pool::ConnectionPool, models::bonus_points_log::BonusPointsLogAction};
use arcadia_common::error::Result;
use sqlx::{Postgres, Transaction};

impl ConnectionPool {
    pub async fn purchase_upload(
        &self,
        user_id: i32,
        bytes: i64,
        bonus_points_cost: i64,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        Self::decrement_bonus_points_and_freeleech_tokens(&mut tx, user_id, bonus_points_cost, 0)
            .await?;
        Self::add_upload(&mut tx, user_id, bytes).await?;
        let upload_details = format!("{bytes} bytes");
        Self::log_bonus_points_change_tx(
            &mut tx,
            user_id,
            BonusPointsLogAction::ShopPurchaseUpload,
            -bonus_points_cost,
            Some(&upload_details),
            None,
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn purchase_freeleech_tokens(
        &self,
        user_id: i32,
        quantity: i32,
        bonus_points_cost: i64,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        Self::decrement_bonus_points_and_freeleech_tokens(&mut tx, user_id, bonus_points_cost, 0)
            .await?;
        Self::add_freeleech_tokens(&mut tx, user_id, quantity).await?;
        let token_details = format!("{quantity} freeleech tokens");
        Self::log_bonus_points_change_tx(
            &mut tx,
            user_id,
            BonusPointsLogAction::ShopPurchaseFreeleechTokens,
            -bonus_points_cost,
            Some(&token_details),
            None,
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn add_upload(
        tx: &mut Transaction<'_, Postgres>,
        user_id: i32,
        bytes: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE users
                SET uploaded = uploaded + $1
                WHERE id = $2
            "#,
            bytes,
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn add_freeleech_tokens(
        tx: &mut Transaction<'_, Postgres>,
        user_id: i32,
        quantity: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE users
                SET freeleech_tokens = freeleech_tokens + $1
                WHERE id = $2
            "#,
            quantity,
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
