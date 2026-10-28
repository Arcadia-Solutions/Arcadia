use crate::{
    connection_pool::ConnectionPool,
    models::{
        arcadia_settings::TorrentRequestVoteCurrency,
        artist::AffiliatedArtistLite,
        bonus_points_log::BonusPointsLogAction,
        common::PaginatedResults,
        torrent_request::{
            EditedTorrentRequest, TorrentRequest, TorrentRequestDeletionKind,
            TorrentRequestWithTitleGroupLite, UserCreatedTorrentRequest,
        },
    },
};
use arcadia_common::error::{Error, Result};
use arcadia_shared::utils::format_title_group_name;
use chrono::{Duration, Utc};
use serde_json::Value;
use sqlx::{query_as, Postgres, Transaction};
use std::collections::HashMap;

impl ConnectionPool {
    pub async fn fetch_title_group_name_for_torrent_request(
        tx: &mut Transaction<'_, Postgres>,
        torrent_request_id: i64,
    ) -> Result<String> {
        let row = sqlx::query!(
            r#"
            SELECT
                tg.name AS "name!",
                s.name AS "series_name?"
            FROM torrent_requests tr
            JOIN title_groups tg ON tg.id = tr.title_group_id
            LEFT JOIN series s ON s.id = tg.series_id
            WHERE tr.id = $1
            "#,
            torrent_request_id
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(format_title_group_name(
            row.series_name.as_deref(),
            &row.name,
        ))
    }

    pub async fn create_torrent_request(
        &self,
        torrent_request: &mut UserCreatedTorrentRequest,
        user_id: i32,
        vote_currencies: &[TorrentRequestVoteCurrency],
    ) -> Result<TorrentRequest> {
        //TODO: make those requests transactional
        let create_torrent_request_query = r#"
            INSERT INTO torrent_requests
            (title_group_id, created_by_id, edition_name,
            release_group, description, languages, container, audio_codec,
            audio_channels, video_codec, features, subtitle_languages, video_resolution,
            audio_bitrate_sampling, source)
            VALUES ($1, $2, $3, $4, $5, $6::language_enum[], $7, $8::audio_codec_enum[], $9::audio_channels_enum[],
            $10::video_codec_enum[], $11::features_enum[], $12::language_enum[], $13::video_resolution_enum[],
            $14::audio_bitrate_sampling_enum[], $15::source_enum[])
            RETURNING id, title_group_id, edition_name, release_group, created_at, updated_at, created_by_id, description, languages, container, audio_codec, audio_channels, video_codec, features, subtitle_languages, video_resolution, video_resolution_other_x, video_resolution_other_y, audio_bitrate_sampling, source, filled_by_user_id, filled_by_torrent_id, filled_at;
        "#;

        let created_torrent_request =
            sqlx::query_as::<_, TorrentRequest>(create_torrent_request_query)
                .bind(torrent_request.title_group_id)
                .bind(user_id)
                .bind(&torrent_request.edition_name)
                .bind(&torrent_request.release_group)
                .bind(&torrent_request.description)
                .bind(&torrent_request.languages)
                .bind(&torrent_request.container)
                .bind(&torrent_request.audio_codec)
                .bind(&torrent_request.audio_channels)
                .bind(&torrent_request.video_codec)
                .bind(&torrent_request.features)
                .bind(&torrent_request.subtitle_languages)
                .bind(&torrent_request.video_resolution)
                .bind(&torrent_request.audio_bitrate_sampling)
                .bind(&torrent_request.source)
                .fetch_one(&self.pool)
                .await
                .map_err(Error::CouldNotCreateTorrentRequest)?;

        torrent_request.initial_vote.torrent_request_id = created_torrent_request.id;

        let _ = self
            .create_torrent_request_vote(&torrent_request.initial_vote, user_id, vote_currencies)
            .await?;

        self.create_subscription_torrent_request_comments(created_torrent_request.id, user_id)
            .await?;

        Ok(created_torrent_request)
    }

    /// returns true if the filler is the uploader of the torrent (which then receives the full bounty)
    /// returns false if the filler is not the uploader of the torrent (which then receives half of the bounty)
    pub async fn fill_torrent_request(
        &self,
        torrent_id: i32,
        torrent_request_id: i64,
        current_user_id: i32,
    ) -> Result<bool> {
        const REQUEST_FILL_UPLOADER_ONLY_GRACE_PERIOD_HOURS: i64 = 1;

        let torrent_upload_info = sqlx::query!(
            r#"
            SELECT
                created_by_id,
                created_at
            FROM torrents
            WHERE id = $1
            "#,
            torrent_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::TorrentNotFound)?;

        let is_torrent_in_requested_title_group = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM torrents t
                JOIN edition_groups eg ON t.edition_group_id = eg.id
                JOIN torrent_requests r ON eg.title_group_id = r.title_group_id
                WHERE t.id = $1
                  AND r.id = $2
            )
            "#,
            torrent_id,
            torrent_request_id
        )
        .fetch_one(&self.pool)
        .await?;

        if !is_torrent_in_requested_title_group.unwrap() {
            return Err(Error::TorrentTitleGroupNotMatchingRequestedOne);
        }

        let is_torrent_request_filled = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM torrent_requests tr
                WHERE tr.id = $1 AND tr.filled_at IS NOT NULL
            )
            "#,
            torrent_request_id
        )
        .fetch_one(&self.pool)
        .await?;

        if is_torrent_request_filled.unwrap() {
            return Err(Error::TorrentRequestAlreadyFilled);
        }

        let grace_period_ends_at = torrent_upload_info.created_at
            + Duration::hours(REQUEST_FILL_UPLOADER_ONLY_GRACE_PERIOD_HOURS);

        if current_user_id != torrent_upload_info.created_by_id && Utc::now() < grace_period_ends_at
        {
            return Err(Error::TorrentRequestFillUploaderOnlyWithinFirstHour);
        }

        #[derive(Debug)]
        struct BountySummary {
            total_upload: i64,
            total_bonus: i64,
        }

        let bounty_summary = query_as!(
            BountySummary,
            r#"
            SELECT
                SUM(bounty_upload)::BIGINT AS "total_upload!",
                SUM(bounty_bonus_points)::BIGINT AS "total_bonus!"
            FROM torrent_request_votes
            WHERE torrent_request_id = $1
            "#,
            torrent_request_id
        )
        .fetch_one(&self.pool)
        .await?;

        // Calculate the share for each user (50% each).
        let (upload_share, bonus_share) = if torrent_upload_info.created_by_id == current_user_id {
            (bounty_summary.total_upload, bounty_summary.total_bonus)
        } else {
            (
                bounty_summary.total_upload / 2,
                bounty_summary.total_bonus / 2,
            )
        };

        let torrent_uploader_id = torrent_upload_info.created_by_id;

        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
            UPDATE users
            SET
                uploaded = users.uploaded +
                    CASE
                        WHEN users.id = $1 THEN $3::BIGINT
                        WHEN users.id = $2 THEN $3::BIGINT
                        ELSE 0
                    END,
                bonus_points = users.bonus_points +
                    CASE
                        WHEN users.id = $1 THEN $4::BIGINT
                        WHEN users.id = $2 THEN $4::BIGINT
                        ELSE 0
                    END,
                requests_filled = requests_filled +
                    CASE
                        WHEN users.id = $2 THEN 1
                        ELSE 0
                    END
            WHERE
                users.id IN ($1, $2)
                "#,
            torrent_uploader_id,
            current_user_id,
            upload_share,
            bonus_share
        )
        .execute(&mut *tx)
        .await?;

        if bonus_share > 0 {
            let title_group_name =
                Self::fetch_title_group_name_for_torrent_request(&mut tx, torrent_request_id)
                    .await?;

            if torrent_uploader_id == current_user_id {
                Self::log_bonus_points_change_tx(
                    &mut tx,
                    current_user_id,
                    BonusPointsLogAction::TorrentRequestFillReward,
                    bonus_share,
                    Some(&title_group_name),
                    Some(torrent_request_id),
                )
                .await?;
            } else {
                Self::log_bonus_points_change_tx(
                    &mut tx,
                    torrent_uploader_id,
                    BonusPointsLogAction::TorrentRequestFillReward,
                    bonus_share,
                    Some(&title_group_name),
                    Some(torrent_request_id),
                )
                .await?;
                Self::log_bonus_points_change_tx(
                    &mut tx,
                    current_user_id,
                    BonusPointsLogAction::TorrentRequestFillReward,
                    bonus_share,
                    Some(&title_group_name),
                    Some(torrent_request_id),
                )
                .await?;
            }
        }

        sqlx::query!(
            r#"
            UPDATE torrent_requests tr
            SET
                filled_by_torrent_id = $1,
                filled_by_user_id = $2,
                filled_at = NOW()
            WHERE
                tr.id = $3
                "#,
            torrent_id,
            current_user_id,
            torrent_request_id
        )
        .execute(&mut *tx)
        .await?;

        // Get the request creator and all distinct voters to notify them
        let recipients: Vec<i32> = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT user_id AS "user_id!"
            FROM (
                SELECT created_by_id AS user_id FROM torrent_requests WHERE id = $1
                UNION
                SELECT created_by_id AS user_id FROM torrent_request_votes WHERE torrent_request_id = $1
            ) AS recipients
            "#,
            torrent_request_id
        )
        .fetch_all(&mut *tx)
        .await?;

        // Send notification messages to all recipients from user id 1
        let message_content = format!(
            "Your [url=/torrent-request/{}]torrent request[/url] has been filled!

            You can see the torrent that filled it [url=/torrent/{}]here[/url]",
            torrent_request_id, torrent_id
        );

        Self::send_batch_messages_tx(
            &mut tx,
            1,
            &recipients,
            "Your torrent request has been filled",
            &message_content,
            true,
        )
        .await?;

        tx.commit().await?;

        Ok(torrent_upload_info.created_by_id == current_user_id)
    }

    /// Deletes the torrent request (as well as its votes and comments), optionally refunding
    /// the bounty to the voters, and notifies every voter except the user doing the deletion.
    pub async fn delete_torrent_request(
        &self,
        torrent_request_id: i64,
        current_user_id: i32,
        deletion_kind: TorrentRequestDeletionKind<'_>,
    ) -> Result<()> {
        let (refund_bounty, deletion_message) = match deletion_kind {
            TorrentRequestDeletionKind::Author => (false, None),
            TorrentRequestDeletionKind::WithPermission {
                refund_bounty,
                message,
            } => (refund_bounty, message),
        };

        let mut tx = self.pool.begin().await?;

        // Locking the request makes the checks below reliable: a concurrent vote creation has to
        // lock the same row (foreign key), and so does a concurrent fill or deletion
        let locked_torrent_request = sqlx::query!(
            r#"
            SELECT filled_at
            FROM torrent_requests
            WHERE id = $1
            FOR UPDATE
            "#,
            torrent_request_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(Error::TorrentRequestNotFound)?;

        if locked_torrent_request.filled_at.is_some() {
            return Err(Error::TorrentRequestAlreadyFilled);
        }

        if matches!(deletion_kind, TorrentRequestDeletionKind::Author) {
            let has_votes_from_other_users = sqlx::query_scalar!(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM torrent_request_votes
                    WHERE torrent_request_id = $1 AND created_by_id != $2
                ) AS "has_votes_from_other_users!"
                "#,
                torrent_request_id,
                current_user_id
            )
            .fetch_one(&mut *tx)
            .await?;

            if has_votes_from_other_users {
                return Err(Error::TorrentRequestHasVotesFromOtherUsers);
            }
        }

        let title_group_name =
            Self::fetch_title_group_name_for_torrent_request(&mut tx, torrent_request_id).await?;

        #[derive(Debug)]
        struct VoterBounty {
            created_by_id: i32,
            bounty_bonus_points: i64,
        }

        let voters = query_as!(
            VoterBounty,
            r#"
            SELECT
                created_by_id AS "created_by_id!",
                SUM(bounty_bonus_points)::BIGINT AS "bounty_bonus_points!"
            FROM torrent_request_votes
            WHERE torrent_request_id = $1
            GROUP BY created_by_id
            "#,
            torrent_request_id
        )
        .fetch_all(&mut *tx)
        .await?;

        // The votes don't exist anymore, so the voters lose the associated counter,
        // and get their bounty back when the deletion refunds it
        sqlx::query!(
            r#"
            UPDATE users u
            SET
                requests_voted = GREATEST(u.requests_voted - 1, 0),
                uploaded = u.uploaded + CASE WHEN $2 THEN v.bounty_upload ELSE 0 END,
                bonus_points = u.bonus_points + CASE WHEN $2 THEN v.bounty_bonus_points ELSE 0 END
            FROM (
                SELECT
                    created_by_id,
                    SUM(bounty_upload) AS bounty_upload,
                    SUM(bounty_bonus_points) AS bounty_bonus_points
                FROM torrent_request_votes
                WHERE torrent_request_id = $1
                GROUP BY created_by_id
            ) v
            WHERE u.id = v.created_by_id
            "#,
            torrent_request_id,
            refund_bounty
        )
        .execute(&mut *tx)
        .await?;

        if refund_bounty {
            for voter in &voters {
                if voter.bounty_bonus_points > 0 {
                    Self::log_bonus_points_change_tx(
                        &mut tx,
                        voter.created_by_id,
                        BonusPointsLogAction::TorrentRequestVoteRefund,
                        voter.bounty_bonus_points,
                        Some(&title_group_name),
                        Some(torrent_request_id),
                    )
                    .await?;
                }
            }
        }

        // the comments are deleted along with the request, so their authors lose the associated counter
        sqlx::query!(
            r#"
            UPDATE users u
            SET request_comments = GREATEST(u.request_comments - c.comments_amount, 0)
            FROM (
                SELECT created_by_id, COUNT(*)::INTEGER AS comments_amount
                FROM torrent_request_comments
                WHERE torrent_request_id = $1
                GROUP BY created_by_id
            ) c
            WHERE u.id = c.created_by_id
            "#,
            torrent_request_id
        )
        .execute(&mut *tx)
        .await?;

        let created_by_id = sqlx::query_scalar!(
            r#"
            DELETE FROM torrent_requests
            WHERE id = $1
            RETURNING created_by_id
            "#,
            torrent_request_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|error| Error::ErrorWhileDeletingTorrentRequest(error.to_string()))?;

        let mut recipients: Vec<i32> = voters.iter().map(|voter| voter.created_by_id).collect();
        if !recipients.contains(&created_by_id) {
            recipients.push(created_by_id);
        }
        recipients.retain(|recipient_id| *recipient_id != current_user_id);

        if !recipients.is_empty() {
            let bounty_information = if refund_bounty {
                "Your bounty has been refunded."
            } else {
                "Your bounty has not been refunded."
            };
            let deletor_message = match deletion_message
                .map(str::trim)
                .filter(|message| !message.is_empty())
            {
                Some(message) => format!(
                    "\n\nMessage from the person who handled the deletion: {}",
                    message
                ),
                None => String::new(),
            };
            let message_content = format!(
                "The torrent request for [b]{}[/b] you voted on has been deleted.\n\n{}{}",
                title_group_name, bounty_information, deletor_message
            );

            Self::send_batch_messages_tx(
                &mut tx,
                1,
                &recipients,
                "A torrent request you voted on has been deleted",
                &message_content,
                true,
            )
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn find_torrent_request(&self, torrent_request_id: i64) -> Result<TorrentRequest> {
        let torrent_request = sqlx::query_as!(
            TorrentRequest,
            r#"
            SELECT
                id, title_group_id, edition_name, release_group,
                created_at, updated_at, created_by_id, description,
                languages AS "languages!: _", container, audio_codec AS "audio_codec: _",
                audio_channels AS "audio_channels: _", video_codec AS "video_codec: _", features AS "features!: _", subtitle_languages AS "subtitle_languages!: _", video_resolution AS "video_resolution!: _",
                video_resolution_other_x, video_resolution_other_y,
                audio_bitrate_sampling AS "audio_bitrate_sampling!: _", source AS "source!: _",
                filled_by_user_id, filled_by_torrent_id, filled_at
            FROM torrent_requests
            WHERE id = $1
            "#,
            torrent_request_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::TorrentRequestNotFound)?;

        Ok(torrent_request)
    }
    pub async fn search_torrent_requests(
        &self,
        query: &crate::models::torrent_request::SearchTorrentRequestsQuery,
        subscribed_user_id: Option<i32>,
    ) -> Result<PaginatedResults<TorrentRequestWithTitleGroupLite>> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(50);
        let offset = (page - 1).max(0) * page_size;
        let order_by = query.order_by.to_string();
        let order_by_direction = query.order_by_direction.to_string();
        let include_filled = query.include_filled;

        // Get total count
        let total_items = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM torrent_requests tr
            JOIN title_groups tg ON tr.title_group_id = tg.id
            WHERE ($1::TEXT IS NULL OR tg.name ILIKE '%' || $1 || '%' OR EXISTS (SELECT 1 FROM unnest(tg.name_aliases) alias WHERE alias ILIKE '%' || $1 || '%'))
              AND ($2 OR tr.filled_at IS NULL)
              AND ($3::INT IS NULL OR EXISTS (
                  SELECT 1 FROM subscriptions_torrent_request_comments strc
                  WHERE strc.torrent_request_id = tr.id AND strc.user_id = $3
              ))
            "#,
            query.title_group_name,
            include_filled,
            subscribed_user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotSearchForTorrentRequests)?
        .unwrap_or(0);

        // First, fetch the main torrent request data
        #[derive(Debug)]
        struct TorrentRequestRow {
            // torrent_request fields
            tr_id: i64,
            tr_title_group_id: i32,
            tr_created_at: chrono::DateTime<chrono::Utc>,
            tr_updated_at: chrono::DateTime<chrono::Utc>,
            tr_created_by_id: i32,
            tr_filled_by_user_id: Option<i32>,
            tr_filled_by_torrent_id: Option<i32>,
            tr_filled_at: Option<chrono::DateTime<chrono::Utc>>,
            tr_edition_name: Option<String>,
            tr_source: Vec<crate::models::edition_group::Source>,
            tr_release_group: Option<String>,
            tr_description: Option<String>,
            tr_languages: Vec<crate::models::torrent::Language>,
            tr_container: Vec<String>,
            tr_audio_codec: Vec<crate::models::torrent::AudioCodec>,
            tr_audio_channels: Vec<crate::models::torrent::AudioChannels>,
            tr_audio_bitrate_sampling: Vec<crate::models::torrent::AudioBitrateSampling>,
            tr_video_codec: Vec<crate::models::torrent::VideoCodec>,
            tr_features: Vec<crate::models::torrent::Features>,
            tr_subtitle_languages: Vec<crate::models::torrent::Language>,
            tr_video_resolution: Vec<crate::models::torrent::VideoResolution>,
            tr_video_resolution_other_x: Option<i32>,
            tr_video_resolution_other_y: Option<i32>,
            // created_by user fields
            created_by_id: i32,
            created_by_username: String,
            created_by_warned: bool,
            created_by_banned: bool,
            // filled_by user fields
            filled_by_id: Option<i32>,
            filled_by_username: Option<String>,
            filled_by_warned: Option<bool>,
            filled_by_banned: Option<bool>,
            // title_group fields
            tg_id: i32,
            tg_name: String,
            tg_content_type: crate::models::title_group::ContentType,
            tg_original_release_date: Option<chrono::NaiveDate>,
            tg_original_release_date_only_year_known: bool,
            tg_covers: Vec<String>,
            tg_platform: Option<crate::models::title_group::Platform>,
            // bounty fields
            bounty_upload: i64,
            bounty_bonus_points: i64,
            // votes
            user_votes_amount: i64,
            // series
            series_id: Option<i64>,
            series_name: Option<String>,
        }

        let rows = sqlx::query_as!(
            TorrentRequestRow,
            r#"
            SELECT
                tr.id AS tr_id,
                tr.title_group_id AS tr_title_group_id,
                tr.created_at AS tr_created_at,
                tr.updated_at AS tr_updated_at,
                tr.created_by_id AS tr_created_by_id,
                tr.filled_by_user_id AS tr_filled_by_user_id,
                tr.filled_by_torrent_id AS tr_filled_by_torrent_id,
                tr.filled_at AS tr_filled_at,
                tr.edition_name AS tr_edition_name,
                tr.source AS "tr_source!: _",
                tr.release_group AS tr_release_group,
                tr.description AS tr_description,
                tr.languages AS "tr_languages!: _",
                tr.container AS "tr_container!",
                tr.audio_codec AS "tr_audio_codec!: _",
                tr.audio_channels AS "tr_audio_channels!: _",
                tr.audio_bitrate_sampling AS "tr_audio_bitrate_sampling!: _",
                tr.video_codec AS "tr_video_codec!: _",
                tr.features AS "tr_features!: _",
                tr.subtitle_languages AS "tr_subtitle_languages!: _",
                tr.video_resolution AS "tr_video_resolution!: _",
                tr.video_resolution_other_x AS tr_video_resolution_other_x,
                tr.video_resolution_other_y AS tr_video_resolution_other_y,
                u.id AS "created_by_id!",
                u.username AS "created_by_username!",
                u.warned AS "created_by_warned!",
                u.banned AS "created_by_banned!",
                filled_by_user.id AS "filled_by_id?",
                filled_by_user.username AS "filled_by_username?",
                filled_by_user.warned AS "filled_by_warned?",
                filled_by_user.banned AS "filled_by_banned?",
                tg.id AS tg_id,
                tg.name AS tg_name,
                tg.content_type AS "tg_content_type!: _",
                tg.original_release_date AS tg_original_release_date,
                tg.original_release_date_only_year_known AS tg_original_release_date_only_year_known,
                tg.covers AS "tg_covers!",
                tg.platform AS "tg_platform: _",
                COALESCE(
                    (SELECT SUM(trv.bounty_upload)::BIGINT
                     FROM torrent_request_votes trv
                     WHERE trv.torrent_request_id = tr.id),
                    0::BIGINT
                ) AS "bounty_upload!",
                COALESCE(
                    (SELECT SUM(trv.bounty_bonus_points)::BIGINT
                     FROM torrent_request_votes trv
                     WHERE trv.torrent_request_id = tr.id),
                    0::BIGINT
                ) AS "bounty_bonus_points!",
                COALESCE(
                    (SELECT COUNT(DISTINCT trv2.created_by_id)::BIGINT
                     FROM torrent_request_votes trv2
                     WHERE trv2.torrent_request_id = tr.id),
                    0::BIGINT
                ) AS "user_votes_amount!",
                s.id AS "series_id?",
                s.name AS "series_name?"
            FROM torrent_requests tr
            JOIN title_groups tg ON tr.title_group_id = tg.id
            JOIN users u ON u.id = tr.created_by_id
            LEFT JOIN users filled_by_user ON filled_by_user.id = tr.filled_by_user_id
            LEFT JOIN series s ON s.id = tg.series_id
            WHERE ($1::TEXT IS NULL OR tg.name ILIKE '%' || $1 || '%' OR EXISTS (SELECT 1 FROM unnest(tg.name_aliases) alias WHERE alias ILIKE '%' || $1 || '%'))
              AND ($4 OR tr.filled_at IS NULL)
              AND ($7::INT IS NULL OR EXISTS (
                  SELECT 1 FROM subscriptions_torrent_request_comments strc
                  WHERE strc.torrent_request_id = tr.id AND strc.user_id = $7
              ))
            ORDER BY
                CASE WHEN $5 = 'upload' AND $6 = 'asc' THEN (SELECT COALESCE(SUM(trv.bounty_upload), 0) FROM torrent_request_votes trv WHERE trv.torrent_request_id = tr.id) END ASC,
                CASE WHEN $5 = 'upload' AND $6 = 'desc' THEN (SELECT COALESCE(SUM(trv.bounty_upload), 0) FROM torrent_request_votes trv WHERE trv.torrent_request_id = tr.id) END DESC,
                CASE WHEN $5 = 'bonus_points' AND $6 = 'asc' THEN (SELECT COALESCE(SUM(trv.bounty_bonus_points), 0) FROM torrent_request_votes trv WHERE trv.torrent_request_id = tr.id) END ASC,
                CASE WHEN $5 = 'bonus_points' AND $6 = 'desc' THEN (SELECT COALESCE(SUM(trv.bounty_bonus_points), 0) FROM torrent_request_votes trv WHERE trv.torrent_request_id = tr.id) END DESC,
                CASE WHEN $5 = 'voters' AND $6 = 'asc' THEN (SELECT COALESCE(COUNT(DISTINCT trv.created_by_id), 0) FROM torrent_request_votes trv WHERE trv.torrent_request_id = tr.id) END ASC,
                CASE WHEN $5 = 'voters' AND $6 = 'desc' THEN (SELECT COALESCE(COUNT(DISTINCT trv.created_by_id), 0) FROM torrent_request_votes trv WHERE trv.torrent_request_id = tr.id) END DESC,
                CASE WHEN $5 = 'created_at' AND $6 = 'asc' THEN tr.created_at END ASC,
                CASE WHEN $5 = 'created_at' AND $6 = 'desc' THEN tr.created_at END DESC
            LIMIT $2 OFFSET $3
            "#,
            query.title_group_name,
            page_size,
            offset,
            include_filled,
            order_by,
            order_by_direction,
            subscribed_user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotSearchForTorrentRequests)?;

        // Get title group IDs for affiliated artists query
        let title_group_ids: Vec<i32> = rows.iter().map(|r| r.tg_id).collect();

        // Fetch affiliated artists (similar to torrent search pattern)
        let affiliated_artists = if title_group_ids.is_empty() {
            vec![]
        } else {
            sqlx::query!(
                r#"
                SELECT
                    aa.title_group_id,
                    a.id as artist_id,
                    a.name as artist_name
                FROM affiliated_artists aa
                JOIN artists a ON a.id = aa.artist_id
                WHERE aa.title_group_id = ANY($1)
                "#,
                &title_group_ids
            )
            .fetch_all(&self.pool)
            .await?
        };

        // Group artists by title_group_id
        let mut grouped_artists: HashMap<i32, Vec<AffiliatedArtistLite>> = HashMap::new();
        for row in affiliated_artists {
            grouped_artists
                .entry(row.title_group_id)
                .or_default()
                .push(AffiliatedArtistLite {
                    artist_id: row.artist_id,
                    name: row.artist_name,
                });
        }

        let results = rows
            .into_iter()
            .map(|row| {
                let series = match (row.series_id, row.series_name.clone()) {
                    (Some(id), Some(name)) => Some(crate::models::series::SeriesLite { id, name }),
                    _ => None,
                };

                TorrentRequestWithTitleGroupLite {
                    torrent_request: crate::models::torrent_request::TorrentRequestHierarchyLite {
                        torrent_request: TorrentRequest {
                            id: row.tr_id,
                            title_group_id: row.tr_title_group_id,
                            created_at: row.tr_created_at,
                            updated_at: row.tr_updated_at,
                            created_by_id: row.tr_created_by_id,
                            filled_by_user_id: row.tr_filled_by_user_id,
                            filled_by_torrent_id: row.tr_filled_by_torrent_id,
                            filled_at: row.tr_filled_at,
                            edition_name: row.tr_edition_name,
                            source: row.tr_source,
                            release_group: row.tr_release_group,
                            description: row.tr_description,
                            languages: row.tr_languages,
                            container: row.tr_container,
                            audio_codec: row.tr_audio_codec,
                            audio_channels: row.tr_audio_channels,
                            audio_bitrate_sampling: row.tr_audio_bitrate_sampling,
                            video_codec: row.tr_video_codec,
                            features: row.tr_features,
                            subtitle_languages: row.tr_subtitle_languages,
                            video_resolution: row.tr_video_resolution,
                            video_resolution_other_x: row.tr_video_resolution_other_x,
                            video_resolution_other_y: row.tr_video_resolution_other_y,
                        },
                        created_by: crate::models::user::UserLite {
                            id: row.created_by_id,
                            username: row.created_by_username,
                            warned: row.created_by_warned,
                            banned: row.created_by_banned,
                        },
                        filled_by: row.filled_by_id.map(|id| crate::models::user::UserLite {
                            id,
                            username: row.filled_by_username.clone().unwrap_or_default(),
                            warned: row.filled_by_warned.unwrap_or(false),
                            banned: row.filled_by_banned.unwrap_or(false),
                        }),
                        bounty: crate::models::torrent_request::TorrentRequestBounty {
                            bonus_points: row.bounty_bonus_points,
                            upload: row.bounty_upload,
                        },
                        user_votes_amount: row.user_votes_amount as i32,
                    },
                    title_group: crate::models::title_group::TitleGroupLite {
                        id: row.tg_id,
                        name: row.tg_name,
                        content_type: row.tg_content_type,
                        original_release_date: row.tg_original_release_date,
                        original_release_date_only_year_known: row
                            .tg_original_release_date_only_year_known,
                        covers: row.tg_covers,
                        edition_groups: vec![],
                        platform: row.tg_platform,
                        series: series.clone(),
                        latest_torrent_uploaded_by: None,
                        latest_torrent_uploaded_at: None,
                    },
                    affiliated_artists: grouped_artists
                        .get(&row.tg_id)
                        .cloned()
                        .unwrap_or_default(),
                    series,
                }
            })
            .collect();

        Ok(PaginatedResults {
            results,
            page: page as u32,
            page_size: page_size as u32,
            total_items,
        })
    }

    pub async fn find_torrent_request_hierarchy(
        &self,
        torrent_request_id: i64,
        current_user_id: i32,
    ) -> Result<Value> {
        let result = sqlx::query!(
            r#"
        SELECT json_build_object(
                    'torrent_request', tr,
                    'title_group', tg,
                    'affiliated_artists', COALESCE((
                        SELECT json_agg(
                            json_build_object(
                                'id', aa.id,
                                'title_group_id', aa.title_group_id,
                                'artist_id', aa.artist_id,
                                'roles', aa.roles,
                                'nickname', aa.nickname,
                                'created_at', aa.created_at,
                                'created_by_id', aa.created_by_id,
                                'artist', json_build_object(
                                    'id', a.id,
                                    'name', a.name,
                                    'created_at', a.created_at,
                                    'created_by_id', a.created_by_id,
                                    'description', a.description,
                                    'pictures', a.pictures,
                                    'title_groups_amount', a.title_groups_amount,
                                    'edition_groups_amount', a.edition_groups_amount,
                                    'torrents_amount', a.torrents_amount,
                                    'seeders_amount', a.seeders_amount,
                                    'leechers_amount', a.leechers_amount,
                                    'snatches_amount', a.snatches_amount
                                )
                            )
                        )
                        FROM affiliated_artists aa
                        JOIN artists a ON a.id = aa.artist_id
                        WHERE aa.title_group_id = tg.id
                    ), '[]'::json),
                    'series', COALESCE((
                        SELECT json_build_object('id', s.id, 'name', s.name)
                        FROM series s
                        WHERE s.id = tg.series_id
                    ), '{}'::json),
                    'votes', COALESCE((
                        SELECT json_agg(
                            json_build_object(
                                'id', trv3.id,
                                'torrent_request_id', trv3.torrent_request_id,
                                'created_at', trv3.created_at,
                                'created_by_id', trv3.created_by_id,
                                'created_by', json_build_object(
                                    'id', u.id,
                                    'username', u.username,
                                    'warned', u.warned,
                                    'banned', u.banned
                                ),
                                'bounty_upload', trv3.bounty_upload,
                                'bounty_bonus_points', trv3.bounty_bonus_points
                            )
                            ORDER BY trv3.created_at DESC
                        )
                        FROM torrent_request_votes trv3
                        JOIN users u ON u.id = trv3.created_by_id
                        WHERE trv3.torrent_request_id = tr.id
                    ), '[]'::json),
                    'comments', COALESCE((
                        SELECT json_agg(
                            json_build_object(
                                'id', trc.id,
                                'torrent_request_id', trc.torrent_request_id,
                                'created_by_id', trc.created_by_id,
                                'created_by', json_build_object(
                                    'id', u2.id,
                                    'username', u2.username,
                                    'class_name', u2.class_name,
                                    'custom_title', u2.custom_title,
                                    'warned', u2.warned,
                                    'banned', u2.banned,
                                    'avatar', u2.avatar
                                ),
                                'content', trc.content,
                                'created_at', trc.created_at,
                                'updated_at', trc.updated_at
                            )
                            ORDER BY trc.created_at ASC
                        )
                        FROM torrent_request_comments trc
                        JOIN users u2 ON u2.id = trc.created_by_id
                        WHERE trc.torrent_request_id = tr.id
                    ), '[]'::json),
                    'filled_by_user', (
                        SELECT json_build_object(
                            'id', u3.id,
                            'username', u3.username,
                            'warned', u3.warned,
                            'banned', u3.banned
                        )
                        FROM users u3
                        WHERE u3.id = tr.filled_by_user_id
                    ),
                    'is_subscribed_to_comments', EXISTS(
                        SELECT 1
                        FROM subscriptions_torrent_request_comments strc
                        WHERE strc.torrent_request_id = tr.id
                        AND strc.user_id = $2
                    )
                ) as data
                FROM torrent_requests tr
                JOIN title_groups tg ON tr.title_group_id = tg.id
                WHERE tr.id = $1
        "#,
            torrent_request_id,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindTheTorrentRequest)?;

        Ok(result.data.unwrap())
    }

    pub async fn update_torrent_request(
        &self,
        edited_torrent_request: &EditedTorrentRequest,
        torrent_request_id: i64,
    ) -> Result<TorrentRequest> {
        let updated_torrent_request = sqlx::query_as!(
            TorrentRequest,
            r#"
            UPDATE torrent_requests
            SET
                title_group_id = $2,
                edition_name = $3,
                release_group = $4,
                description = $5,
                languages = $6,
                container = $7,
                audio_codec = $8,
                audio_channels = $9,
                video_codec = $10,
                features = $11,
                subtitle_languages = $12,
                video_resolution = $13,
                audio_bitrate_sampling = $14,
                source = $15,
                updated_at = NOW()
            WHERE id = $1
            -- the bounty was voted for this title group, so it can not be redirected to another one
            AND (
                title_group_id = $2
                OR (
                    filled_at IS NULL
                    AND NOT EXISTS (
                        SELECT 1 FROM torrent_request_votes
                        WHERE torrent_request_id = $1 AND created_by_id != torrent_requests.created_by_id
                    )
                )
            )
            RETURNING
                id, title_group_id, edition_name, release_group,
                created_at, updated_at, created_by_id, description,
                languages AS "languages!: _", container, audio_codec AS "audio_codec: _",
                audio_channels AS "audio_channels: _", video_codec AS "video_codec: _", features AS "features!: _", subtitle_languages AS "subtitle_languages!: _", video_resolution AS "video_resolution!: _",
                video_resolution_other_x, video_resolution_other_y,
                audio_bitrate_sampling AS "audio_bitrate_sampling!: _", source AS "source!: _",
                filled_by_user_id, filled_by_torrent_id, filled_at
            "#,
            torrent_request_id,
            edited_torrent_request.title_group_id,
            edited_torrent_request.edition_name,
            edited_torrent_request.release_group,
            edited_torrent_request.description,
            edited_torrent_request.languages as _,
            &edited_torrent_request.container,
            edited_torrent_request.audio_codec as _,
            edited_torrent_request.audio_channels as _,
            edited_torrent_request.video_codec as _,
            edited_torrent_request.features as _,
            edited_torrent_request.subtitle_languages as _,
            edited_torrent_request.video_resolution as _,
            edited_torrent_request.audio_bitrate_sampling as _,
            edited_torrent_request.source as _,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::ErrorWhileUpdatingTorrentRequest(e.to_string()))?
        .ok_or(Error::TorrentRequestTitleGroupLocked)?;

        Ok(updated_torrent_request)
    }
}
