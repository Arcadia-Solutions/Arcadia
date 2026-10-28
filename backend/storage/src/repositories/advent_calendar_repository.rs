use crate::{
    connection_pool::ConnectionPool,
    models::{
        advent_calendar::{
            AdventCalendar, AdventCalendarDoor, AdventCalendarDoorOpening,
            AdventCalendarDoorOpeningResult, AdventCalendarDoorSettings,
            AdventCalendarLateOpeningPolicy, AdventCalendarLite, AdventCalendarRewardDistribution,
            UserCreatedAdventCalendar,
        },
        bonus_points_log::BonusPointsLogAction,
    },
    services::advent_calendar_service::check_advent_calendar_door_openable,
};
use arcadia_common::error::{Error, Result};
use chrono::{DateTime, Utc};
use sqlx::{types::Json, PgExecutor, Postgres, Transaction};

fn map_advent_calendar_write_error(error: sqlx::Error) -> Error {
    if let sqlx::Error::Database(database_error) = &error
        && database_error.code().as_deref() == Some("23505")
        && database_error.constraint() == Some("advent_calendars_single_published")
    {
        return Error::AdventCalendarAlreadyPublished;
    }
    error.into()
}

impl ConnectionPool {
    pub async fn find_advent_calendars_lite(&self) -> Result<Vec<AdventCalendarLite>> {
        let calendars = sqlx::query_as!(
            AdventCalendarLite,
            r#"
                SELECT
                    calendars.id,
                    calendars.name,
                    calendars.starts_at,
                    calendars.published,
                    COUNT(doors.id) as "door_count!",
                    COALESCE(SUM(doors.duration_minutes), 0)::BIGINT as "total_duration_minutes!"
                FROM advent_calendars calendars
                LEFT JOIN advent_calendar_doors doors ON doors.calendar_id = calendars.id
                GROUP BY calendars.id
                ORDER BY calendars.starts_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(calendars)
    }

    pub async fn find_advent_calendar(&self, id: i32) -> Result<AdventCalendar> {
        Self::find_advent_calendar_with_executor(&self.pool, id).await
    }

    async fn find_advent_calendar_with_executor<'connection>(
        executor: impl PgExecutor<'connection>,
        id: i32,
    ) -> Result<AdventCalendar> {
        let calendar = sqlx::query_as!(
            AdventCalendar,
            r#"
                SELECT
                    calendars.id,
                    calendars.created_at,
                    calendars.created_by_id,
                    calendars.name,
                    calendars.starts_at,
                    calendars.top_message,
                    calendars.button_text,
                    calendars.staff_notes,
                    calendars.late_opening_policy as "late_opening_policy: AdventCalendarLateOpeningPolicy",
                    calendars.late_opening_grace_hours,
                    calendars.published,
                    COALESCE(
                        (
                            SELECT jsonb_agg(doors ORDER BY doors.position)
                            FROM advent_calendar_doors doors
                            WHERE doors.calendar_id = calendars.id
                        ),
                        '[]'
                    ) as "doors!: Json<Vec<AdventCalendarDoor>>"
                FROM advent_calendars calendars
                WHERE calendars.id = $1
            "#,
            id
        )
        .fetch_optional(executor)
        .await?
        .ok_or(Error::AdventCalendarNotFound)?;

        Ok(calendar)
    }

    pub async fn create_advent_calendar(
        &self,
        calendar: &UserCreatedAdventCalendar,
        user_id: i32,
    ) -> Result<AdventCalendar> {
        let mut transaction = self.pool.begin().await?;

        let calendar_id = sqlx::query_scalar!(
            r#"
                INSERT INTO advent_calendars (
                    created_by_id, name, starts_at, top_message, button_text, staff_notes,
                    late_opening_policy, late_opening_grace_hours, published
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id
            "#,
            user_id,
            calendar.name,
            calendar.starts_at,
            calendar.top_message,
            calendar.button_text,
            calendar.staff_notes,
            calendar.late_opening_policy as AdventCalendarLateOpeningPolicy,
            calendar.late_opening_grace_hours,
            calendar.published,
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(map_advent_calendar_write_error)?;

        for (position, door) in calendar.doors.iter().enumerate() {
            Self::upsert_advent_calendar_door(
                &mut transaction,
                None,
                calendar_id,
                position as i32,
                &door.settings,
            )
            .await?;
        }

        transaction.commit().await?;

        self.find_advent_calendar(calendar_id).await
    }

    pub async fn update_advent_calendar(
        &self,
        id: i32,
        calendar: &UserCreatedAdventCalendar,
    ) -> Result<AdventCalendar> {
        let mut transaction = self.pool.begin().await?;

        sqlx::query_scalar!(
            r#"
                UPDATE advent_calendars
                SET
                    name = $2,
                    starts_at = $3,
                    top_message = $4,
                    button_text = $5,
                    staff_notes = $6,
                    late_opening_policy = $7,
                    late_opening_grace_hours = $8,
                    published = $9
                WHERE id = $1
                RETURNING id
            "#,
            id,
            calendar.name,
            calendar.starts_at,
            calendar.top_message,
            calendar.button_text,
            calendar.staff_notes,
            calendar.late_opening_policy as AdventCalendarLateOpeningPolicy,
            calendar.late_opening_grace_hours,
            calendar.published,
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_advent_calendar_write_error)?
        .ok_or(Error::AdventCalendarNotFound)?;

        // doors left out of the list are deleted, along with their openings
        let kept_door_ids: Vec<i32> = calendar.doors.iter().filter_map(|door| door.id).collect();
        sqlx::query!(
            r#"
                DELETE FROM advent_calendar_doors
                WHERE calendar_id = $1 AND id <> ALL($2)
            "#,
            id,
            &kept_door_ids
        )
        .execute(&mut *transaction)
        .await?;

        // an unknown id would make the upsert create a door with an id chosen by the client
        let known_door_count = sqlx::query_scalar!(
            r#"
                SELECT COUNT(*) as "count!"
                FROM advent_calendar_doors
                WHERE calendar_id = $1 AND id = ANY($2)
            "#,
            id,
            &kept_door_ids
        )
        .fetch_one(&mut *transaction)
        .await?;
        if known_door_count != kept_door_ids.len() as i64 {
            return Err(Error::AdventCalendarDoorNotFound);
        }

        for (position, door) in calendar.doors.iter().enumerate() {
            Self::upsert_advent_calendar_door(
                &mut transaction,
                door.id,
                id,
                position as i32,
                &door.settings,
            )
            .await?;
        }

        transaction.commit().await?;

        self.find_advent_calendar(id).await
    }

    /// creates the door when it has no id, updates it otherwise
    async fn upsert_advent_calendar_door(
        transaction: &mut Transaction<'_, Postgres>,
        door_id: Option<i32>,
        calendar_id: i32,
        position: i32,
        door: &AdventCalendarDoorSettings,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO advent_calendar_doors (
                    id, calendar_id, position, name, image_url, duration_minutes,
                    opening_message, passphrase, staff_notes,
                    upload_min, upload_max, upload_distribution,
                    download_removed_min, download_removed_max, download_removed_distribution,
                    freeleech_tokens_min, freeleech_tokens_max, freeleech_tokens_distribution,
                    invitations_min, invitations_max, invitations_distribution,
                    bonus_points_min, bonus_points_max, bonus_points_distribution,
                    locked_message
                )
                VALUES (
                    COALESCE($1, nextval(pg_get_serial_sequence('advent_calendar_doors', 'id'))::INT),
                    $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
                    $25
                )
                ON CONFLICT (id) DO UPDATE SET
                    position = EXCLUDED.position,
                    name = EXCLUDED.name,
                    image_url = EXCLUDED.image_url,
                    duration_minutes = EXCLUDED.duration_minutes,
                    opening_message = EXCLUDED.opening_message,
                    passphrase = EXCLUDED.passphrase,
                    staff_notes = EXCLUDED.staff_notes,
                    upload_min = EXCLUDED.upload_min,
                    upload_max = EXCLUDED.upload_max,
                    upload_distribution = EXCLUDED.upload_distribution,
                    download_removed_min = EXCLUDED.download_removed_min,
                    download_removed_max = EXCLUDED.download_removed_max,
                    download_removed_distribution = EXCLUDED.download_removed_distribution,
                    freeleech_tokens_min = EXCLUDED.freeleech_tokens_min,
                    freeleech_tokens_max = EXCLUDED.freeleech_tokens_max,
                    freeleech_tokens_distribution = EXCLUDED.freeleech_tokens_distribution,
                    invitations_min = EXCLUDED.invitations_min,
                    invitations_max = EXCLUDED.invitations_max,
                    invitations_distribution = EXCLUDED.invitations_distribution,
                    bonus_points_min = EXCLUDED.bonus_points_min,
                    bonus_points_max = EXCLUDED.bonus_points_max,
                    bonus_points_distribution = EXCLUDED.bonus_points_distribution,
                    locked_message = EXCLUDED.locked_message
            "#,
            door_id,
            calendar_id,
            position,
            door.name,
            door.image_url,
            door.duration_minutes,
            door.opening_message,
            door.passphrase,
            door.staff_notes,
            door.upload_min,
            door.upload_max,
            door.upload_distribution as AdventCalendarRewardDistribution,
            door.download_removed_min,
            door.download_removed_max,
            door.download_removed_distribution as AdventCalendarRewardDistribution,
            door.freeleech_tokens_min,
            door.freeleech_tokens_max,
            door.freeleech_tokens_distribution as AdventCalendarRewardDistribution,
            door.invitations_min,
            door.invitations_max,
            door.invitations_distribution as AdventCalendarRewardDistribution,
            door.bonus_points_min,
            door.bonus_points_max,
            door.bonus_points_distribution as AdventCalendarRewardDistribution,
            door.locked_message,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete_advent_calendar(&self, id: i32) -> Result<()> {
        let deleted = sqlx::query!("DELETE FROM advent_calendars WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if deleted.rows_affected() == 0 {
            return Err(Error::AdventCalendarNotFound);
        }
        Ok(())
    }

    pub async fn find_published_advent_calendar(&self) -> Result<AdventCalendar> {
        let calendar_id =
            sqlx::query_scalar!("SELECT id FROM advent_calendars WHERE published = TRUE")
                .fetch_optional(&self.pool)
                .await?
                .ok_or(Error::AdventCalendarNotFound)?;

        self.find_advent_calendar(calendar_id).await
    }

    pub async fn find_running_advent_calendar_button_text(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Option<String>> {
        let button_text = sqlx::query_scalar!(
            r#"
                SELECT calendars.button_text
                FROM advent_calendars calendars
                WHERE calendars.published = TRUE
                    AND calendars.starts_at <= $1
                    AND calendars.starts_at + INTERVAL '1 minute' * (
                        SELECT COALESCE(SUM(doors.duration_minutes), 0)
                        FROM advent_calendar_doors doors
                        WHERE doors.calendar_id = calendars.id
                    ) > $1
            "#,
            now
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(button_text)
    }

    pub async fn find_advent_calendar_door_openings_of_user(
        &self,
        user_id: i32,
        calendar_id: i32,
    ) -> Result<Vec<AdventCalendarDoorOpening>> {
        let openings = sqlx::query_as!(
            AdventCalendarDoorOpening,
            r#"
                SELECT
                    openings.door_id,
                    openings.upload,
                    openings.download_removed,
                    openings.freeleech_tokens,
                    openings.invitations,
                    openings.bonus_points
                FROM advent_calendar_door_openings openings
                JOIN advent_calendar_doors doors ON doors.id = openings.door_id
                WHERE openings.user_id = $1 AND doors.calendar_id = $2
            "#,
            user_id,
            calendar_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(openings)
    }

    /// checks that the door can be opened, then rolls its rewards and grants them to the user
    pub async fn open_advent_calendar_door(
        &self,
        user_id: i32,
        door_id: i32,
        given_passphrase: Option<&str>,
    ) -> Result<AdventCalendarDoorOpeningResult> {
        let mut transaction = self.pool.begin().await?;

        // editing, unpublishing or deleting the calendar waits for this lock,
        // so the door cannot change between the check and the opening
        let calendar_id =
            sqlx::query_scalar!("SELECT id FROM advent_calendars WHERE published FOR SHARE")
                .fetch_optional(&mut *transaction)
                .await?
                .ok_or(Error::AdventCalendarDoorNotFound)?;
        let calendar =
            Self::find_advent_calendar_with_executor(&mut *transaction, calendar_id).await?;
        check_advent_calendar_door_openable(calendar, door_id, given_passphrase, Utc::now())?;

        let result = sqlx::query_as!(
            AdventCalendarDoorOpeningResult,
            r#"
                WITH opening AS (
                    INSERT INTO advent_calendar_door_openings (
                        user_id, door_id, upload, download_removed, freeleech_tokens, invitations, bonus_points
                    )
                    SELECT
                        $1,
                        id,
                        roll_advent_calendar_reward(upload_min, upload_max, upload_distribution),
                        roll_advent_calendar_reward(download_removed_min, download_removed_max, download_removed_distribution),
                        roll_advent_calendar_reward(freeleech_tokens_min, freeleech_tokens_max, freeleech_tokens_distribution)::INT,
                        roll_advent_calendar_reward(invitations_min, invitations_max, invitations_distribution)::INT,
                        roll_advent_calendar_reward(bonus_points_min, bonus_points_max, bonus_points_distribution)
                    FROM advent_calendar_doors
                    WHERE id = $2
                    -- the primary key guarantees that concurrent requests only grant the rewards once
                    ON CONFLICT DO NOTHING
                    RETURNING *
                ),
                updated_user AS (
                    UPDATE users
                    SET
                        uploaded = uploaded + opening.upload,
                        downloaded = GREATEST(downloaded - opening.download_removed, 0),
                        freeleech_tokens = users.freeleech_tokens + opening.freeleech_tokens,
                        invitations = users.invitations + opening.invitations,
                        bonus_points = users.bonus_points + opening.bonus_points
                    FROM opening
                    WHERE users.id = opening.user_id
                ),
                bonus_points_log AS (
                    INSERT INTO bonus_points_logs (user_id, action, amount, item_id)
                    SELECT user_id, $3, bonus_points, door_id
                    FROM opening
                    WHERE bonus_points > 0
                )
                SELECT
                    doors.opening_message,
                    opening.upload as "upload!",
                    opening.download_removed as "download_removed!",
                    opening.freeleech_tokens as "freeleech_tokens!",
                    opening.invitations as "invitations!",
                    opening.bonus_points as "bonus_points!"
                FROM opening
                JOIN advent_calendar_doors doors ON doors.id = opening.door_id
            "#,
            user_id,
            door_id,
            BonusPointsLogAction::AdventCalendarDoorReward as BonusPointsLogAction,
        )
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(Error::AdventCalendarDoorAlreadyOpened)?;

        transaction.commit().await?;

        Ok(result)
    }

    pub async fn delete_advent_calendar_door_openings_of_user(
        &self,
        user_id: i32,
        calendar_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM advent_calendar_door_openings openings
                USING advent_calendar_doors doors
                WHERE doors.id = openings.door_id
                    AND openings.user_id = $1
                    AND doors.calendar_id = $2
            "#,
            user_id,
            calendar_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
