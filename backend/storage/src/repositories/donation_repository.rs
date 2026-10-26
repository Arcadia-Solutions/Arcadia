use crate::{
    connection_pool::ConnectionPool,
    models::{
        donation::{
            Donation, DonationSearchResult, EditedDonation, SearchDonationsQuery,
            SearchDonationsResponse, UserCreatedDonation,
        },
        user::UserLiteAvatar,
    },
};
use arcadia_common::error::{Error, Result};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, FromRow)]
struct DBImportDonationSearchResult {
    id: i64,
    donated_by_id: i32,
    donated_by_user_id: i32,
    donated_by_username: String,
    donated_by_class_name: String,
    donated_by_warned: bool,
    donated_by_banned: bool,
    donated_by_avatar: Option<String>,
    donated_by_custom_title: Option<String>,
    donated_at: DateTime<Utc>,
    created_by_id: i32,
    created_by_user_id: i32,
    created_by_username: String,
    created_by_class_name: String,
    created_by_warned: bool,
    created_by_banned: bool,
    created_by_avatar: Option<String>,
    created_by_custom_title: Option<String>,
    created_at: DateTime<Utc>,
    amount: f64,
    note: Option<String>,
}

impl ConnectionPool {
    pub async fn search_donations(
        &self,
        query: &SearchDonationsQuery,
    ) -> Result<SearchDonationsResponse> {
        let offset = ((query.page - 1) * query.page_size) as i64;
        let limit = query.page_size as i64;

        let total_items = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM donations d
            WHERE ($1::INT IS NULL OR d.donated_by_id = $1)
              AND ($2::INT IS NULL OR d.created_by_id = $2)
              AND ($3::FLOAT IS NULL OR d.amount >= $3)
              AND ($4::FLOAT IS NULL OR d.amount <= $4)
              AND ($5::TIMESTAMPTZ IS NULL OR d.donated_at >= $5)
              AND ($6::TIMESTAMPTZ IS NULL OR d.donated_at <= $6)
            "#,
            query.donated_by_id,
            query.created_by_id,
            query.min_amount,
            query.max_amount,
            query.donated_at_start,
            query.donated_at_end
        )
        .fetch_one(&self.pool)
        .await?;

        let aggregates = sqlx::query!(
            r#"
            SELECT
                COALESCE(SUM(d.amount), 0)::FLOAT as "total_amount!",
                COUNT(DISTINCT d.donated_by_id) as "unique_donors!"
            FROM donations d
            WHERE ($1::INT IS NULL OR d.donated_by_id = $1)
              AND ($2::INT IS NULL OR d.created_by_id = $2)
              AND ($3::FLOAT IS NULL OR d.amount >= $3)
              AND ($4::FLOAT IS NULL OR d.amount <= $4)
              AND ($5::TIMESTAMPTZ IS NULL OR d.donated_at >= $5)
              AND ($6::TIMESTAMPTZ IS NULL OR d.donated_at <= $6)
            "#,
            query.donated_by_id,
            query.created_by_id,
            query.min_amount,
            query.max_amount,
            query.donated_at_start,
            query.donated_at_end
        )
        .fetch_one(&self.pool)
        .await?;

        let db_results = sqlx::query_as!(
            DBImportDonationSearchResult,
            r#"
            SELECT
                d.id,
                d.donated_by_id,
                u1.id AS donated_by_user_id,
                u1.username AS donated_by_username,
                u1.class_name AS donated_by_class_name,
                u1.warned AS donated_by_warned,
                u1.banned AS donated_by_banned,
                u1.avatar AS donated_by_avatar,
                u1.custom_title AS donated_by_custom_title,
                d.donated_at,
                d.created_by_id,
                u2.id AS created_by_user_id,
                u2.username AS created_by_username,
                u2.class_name AS created_by_class_name,
                u2.warned AS created_by_warned,
                u2.banned AS created_by_banned,
                u2.avatar AS created_by_avatar,
                u2.custom_title AS created_by_custom_title,
                d.created_at,
                d.amount,
                d.note
            FROM donations d
            JOIN users u1 ON d.donated_by_id = u1.id
            JOIN users u2 ON d.created_by_id = u2.id
            WHERE ($1::INT IS NULL OR d.donated_by_id = $1)
              AND ($2::INT IS NULL OR d.created_by_id = $2)
              AND ($3::FLOAT IS NULL OR d.amount >= $3)
              AND ($4::FLOAT IS NULL OR d.amount <= $4)
              AND ($5::TIMESTAMPTZ IS NULL OR d.donated_at >= $5)
              AND ($6::TIMESTAMPTZ IS NULL OR d.donated_at <= $6)
            ORDER BY
                CASE WHEN $9 = 'donated_at' AND $10 = 'asc' THEN d.donated_at END ASC,
                CASE WHEN $9 = 'donated_at' AND $10 = 'desc' THEN d.donated_at END DESC,
                CASE WHEN $9 = 'created_at' AND $10 = 'asc' THEN d.created_at END ASC,
                CASE WHEN $9 = 'created_at' AND $10 = 'desc' THEN d.created_at END DESC,
                CASE WHEN $9 = 'amount' AND $10 = 'asc' THEN d.amount END ASC,
                CASE WHEN $9 = 'amount' AND $10 = 'desc' THEN d.amount END DESC
            LIMIT $7
            OFFSET $8
            "#,
            query.donated_by_id,
            query.created_by_id,
            query.min_amount,
            query.max_amount,
            query.donated_at_start,
            query.donated_at_end,
            limit,
            offset,
            query.order_by_column.to_string(),
            query.order_by_direction.to_string()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFetchDonations)?;

        let results: Vec<DonationSearchResult> = db_results
            .into_iter()
            .map(|row| DonationSearchResult {
                id: row.id,
                donated_by_id: row.donated_by_id,
                donated_by: UserLiteAvatar {
                    id: row.donated_by_user_id,
                    username: row.donated_by_username,
                    class_name: row.donated_by_class_name,
                    banned: row.donated_by_banned,
                    avatar: row.donated_by_avatar,
                    warned: row.donated_by_warned,
                    custom_title: row.donated_by_custom_title,
                },
                donated_at: row.donated_at,
                created_by_id: row.created_by_id,
                created_by: UserLiteAvatar {
                    id: row.created_by_user_id,
                    username: row.created_by_username,
                    class_name: row.created_by_class_name,
                    banned: row.created_by_banned,
                    avatar: row.created_by_avatar,
                    warned: row.created_by_warned,
                    custom_title: row.created_by_custom_title,
                },
                created_at: row.created_at,
                amount: row.amount,
                note: row.note,
            })
            .collect();

        Ok(SearchDonationsResponse {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items,
            total_amount: aggregates.total_amount,
            unique_donors_count: aggregates.unique_donors,
        })
    }

    pub async fn create_donation(
        &self,
        request: &UserCreatedDonation,
        created_by_id: i32,
    ) -> Result<Donation> {
        let donation = if let Some(donated_at) = request.donated_at {
            sqlx::query_as!(
                Donation,
                r#"
                INSERT INTO donations (donated_by_id, donated_at, created_by_id, amount, note)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, donated_by_id, donated_at, created_by_id, created_at, amount, note
                "#,
                request.donated_by_id,
                donated_at,
                created_by_id,
                request.amount,
                request.note
            )
            .fetch_one(&self.pool)
            .await
            .map_err(Error::CouldNotCreateDonation)?
        } else {
            sqlx::query_as!(
                Donation,
                r#"
                INSERT INTO donations (donated_by_id, created_by_id, amount, note)
                VALUES ($1, $2, $3, $4)
                RETURNING id, donated_by_id, donated_at, created_by_id, created_at, amount, note
                "#,
                request.donated_by_id,
                created_by_id,
                request.amount,
                request.note
            )
            .fetch_one(&self.pool)
            .await
            .map_err(Error::CouldNotCreateDonation)?
        };

        Ok(donation)
    }

    pub async fn find_donation_by_id(&self, id: i64) -> Result<Donation> {
        let donation = sqlx::query_as!(
            Donation,
            r#"
            SELECT id, donated_by_id, donated_at, created_by_id, created_at, amount, note FROM donations
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFetchDonation)?;

        Ok(donation)
    }

    pub async fn update_donation(&self, request: &EditedDonation) -> Result<Donation> {
        let donation = sqlx::query_as!(
            Donation,
            r#"
            UPDATE donations
            SET donated_by_id = $1,
                donated_at = $2,
                amount = $3,
                note = $4
            WHERE id = $5
            RETURNING id, donated_by_id, donated_at, created_by_id, created_at, amount, note
            "#,
            request.donated_by_id,
            request.donated_at,
            request.amount,
            request.note,
            request.id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotUpdateDonation)?;

        Ok(donation)
    }

    pub async fn delete_donation(&self, id: i64) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM donations
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteDonation)?;

        Ok(())
    }
}
