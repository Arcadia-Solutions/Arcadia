use crate::{
    connection_pool::ConnectionPool,
    models::{
        bonus_points_log::BonusPointsLogAction,
        common::{OrderByDirection, PaginatedResults},
        edition_group::{EditionGroupHierarchyLite, Source},
        notification::NotificationEvent,
        peer::PublicPeer,
        title_group::{ContentType, TitleGroupCategory, TitleGroupHierarchyLite},
        torrent::{
            EditedTorrent, Features, Language, Torrent, TorrentHierarchyLite, TorrentSearch,
            TorrentToDelete, TrumpableNotification, UploadedTorrent, VideoResolution,
        },
        torrent_activity::{
            GetTorrentActivitiesQuery, TorrentActivity, TorrentActivityAndTitleGroup,
            TorrentActivityAndUser, TorrentActivityOfTorrentOrderByColumn,
            TorrentActivityOrderByColumn,
        },
        user::UserLite,
    },
    utils::{compute_torrent_info_hash, NormalizedInfoFields},
};
use arcadia_common::{
    error::{Error, Result},
    services::torrent_service::{get_announce_url, looks_like_url},
};
use arcadia_shared::{tracker::models::torrent::InfoHash, utils::format_title_group_name};
use bip_metainfo::{Info, Metainfo, MetainfoBuilder};
use serde_json::{json, Value};
use sqlx::types::Json;
use std::{collections::HashMap, str::FromStr};
use tokio::sync::broadcast;

use chrono::NaiveDate;

/// Subject of the conversation sent to an uploader when their torrent is marked as trumpable.
const TORRENT_MARKED_TRUMPABLE_SUBJECT: &str = "Your torrent has been marked as trumpable";

#[derive(sqlx::FromRow)]
struct TitleGroupInfoLite {
    id: i32,
    name: String,
    series_name: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ReleaseDateInfo {
    title_group_original_release_date: Option<NaiveDate>,
    edition_group_release_date: Option<NaiveDate>,
}

pub struct GetTorrentResult {
    pub title: String,
    pub file_contents: Vec<u8>,
}

#[expect(clippy::too_many_arguments)]
impl ConnectionPool {
    pub async fn create_torrent(
        &self,
        torrent_form: &UploadedTorrent,
        user_id: i32,
        upload_method: &str,
        bonus_points_given_on_upload: i64,
        bonus_points_snatch_cost: i64,
        torrent_max_release_date_allowed: Option<NaiveDate>,
        max_torrent_size_bytes: u64,
        torrent_source_tag: Option<&str>,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<Torrent> {
        let mut tx = self.pool.begin().await?;

        // Check release date cutoff if configured
        if let Some(max_release_date) = torrent_max_release_date_allowed {
            let release_dates = sqlx::query_as!(
                ReleaseDateInfo,
                r#"
                SELECT
                    tg.original_release_date::DATE AS title_group_original_release_date,
                    eg.release_date AS edition_group_release_date
                FROM edition_groups eg
                JOIN title_groups tg ON eg.title_group_id = tg.id
                WHERE eg.id = $1
                "#,
                torrent_form.edition_group_id.0
            )
            .fetch_one(&mut *tx)
            .await?;

            // if the edition has a release date, use it for the cutoff
            // otherwise use the release date of the title group
            let release_date_to_check = release_dates
                .edition_group_release_date
                .or(release_dates.title_group_original_release_date);

            if let Some(release_date) = release_date_to_check
                && release_date > max_release_date
            {
                return Err(Error::ContentReleasedAfterCutoff(
                    max_release_date.to_string(),
                ));
            }
        }

        let create_torrent_query = r#"
            INSERT INTO torrents (
                edition_group_id, created_by_id, release_name, release_group, description,
                file_amount_per_type, uploaded_as_anonymous, upload_method, file_list, mediainfo, trumpable,
                staff_checked, size, duration, audio_codec, audio_bitrate, audio_bitrate_sampling,
                audio_channels, video_codec, features, subtitle_languages, video_resolution,
                video_resolution_other_x, video_resolution_other_y, container, languages, info_hash, info_dict, extras,
                extra_text, bonus_points_snatch_cost
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14,
                $15::audio_codec_enum, $16, $17::audio_bitrate_sampling_enum,
                $18::audio_channels_enum, $19::video_codec_enum, $20::features_enum[],
                $21::language_enum[], $22::video_resolution_enum, $23, $24, $25,
                $26::language_enum[], $27::bytea, $28::bytea, $29::extras_enum[],
                $30, $31
            )
            RETURNING id, info_hash, upload_factor, download_factor, seeders, leechers, times_completed, grabbed, edition_group_id, created_at, updated_at, created_by_id, deleted_at, deleted_by_id, extras, release_name, release_group, description, file_amount_per_type, uploaded_as_anonymous, upload_method, file_list, mediainfo, trumpable, staff_checked, languages, container, size, duration, audio_codec, audio_bitrate, audio_bitrate_sampling, audio_channels, video_codec, features, subtitle_languages, video_resolution, video_resolution_other_x, video_resolution_other_y, extra_text, bonus_points_snatch_cost
        "#;

        let metainfo = Metainfo::from_bytes(&torrent_form.torrent_file.data)
            .map_err(|_| Error::TorrentFileInvalid)?;

        let info = metainfo.info();

        // We cannot trust that the uploader has set the private field properly, so
        // the info hash is recomputed with the private flag forced and the source
        // tag applied, matching the .torrent file served on download.
        let info_hash = compute_torrent_info_hash(info, torrent_source_tag)?;

        // TODO: torrent metadata extraction should be done on the client side
        let parent_folder = info.directory().map(|d| d.to_str().unwrap()).unwrap_or("");
        let files = info
            .files()
            .map(|f| json!({"name": f.path().to_str().unwrap(), "size": f.length()}))
            .collect::<Vec<_>>();

        let file_list = json!({"parent_folder": parent_folder, "files": files});

        let file_amount_per_type = json!(info
            .files()
            .flat_map(|file| file.path().to_str().unwrap().split('.').next_back())
            .fold(std::collections::HashMap::new(), |mut acc, ext| {
                *acc.entry(ext.to_string()).or_insert(0) += 1;
                acc
            }));

        // the size is declared by the uploader and feeds the bonus points rewards, so it is bounded
        let size = metainfo
            .info()
            .files()
            .try_fold(0u64, |total, file| total.checked_add(file.length()))
            .filter(|size| *size <= max_torrent_size_bytes)
            .ok_or(Error::TorrentTooLarge(max_torrent_size_bytes))? as i64;

        let uploaded_torrent = sqlx::query_as::<_, Torrent>(create_torrent_query)
            .bind(torrent_form.edition_group_id.0)
            .bind(user_id)
            .bind(&*torrent_form.release_name.0)
            .bind(torrent_form.release_group.as_deref())
            .bind(torrent_form.description.as_deref())
            .bind(&file_amount_per_type)
            .bind(torrent_form.uploaded_as_anonymous.0)
            .bind(upload_method)
            .bind(&file_list)
            // set mediainfo to None if empty
            .bind(torrent_form.mediainfo.as_deref().and_then(|s| {
                if s.trim().is_empty() {
                    None
                } else {
                    Some(s.trim().to_string())
                }
            }))
            // TODO: check if the torrent is trumpable: via a service ? if this value is not already specified by the uploader
            .bind(torrent_form.trumpable.as_deref())
            .bind(false)
            .bind(size)
            .bind(torrent_form.duration.as_deref())
            .bind(torrent_form.audio_codec.as_deref())
            .bind(torrent_form.audio_bitrate.as_deref())
            .bind(torrent_form.audio_bitrate_sampling.as_deref())
            .bind(torrent_form.audio_channels.as_deref())
            .bind(torrent_form.video_codec.as_deref())
            .bind(
                torrent_form
                    .features
                    .split(',')
                    .filter(|f| !f.is_empty())
                    .map(|f| Features::from_str(f).ok().unwrap())
                    .collect::<Vec<Features>>(),
            )
            .bind(
                torrent_form
                    .subtitle_languages
                    .0
                    .split(',')
                    .filter(|f| !f.is_empty())
                    .map(|f| f.trim())
                    .collect::<Vec<&str>>(),
            )
            .bind(torrent_form.video_resolution.as_deref())
            .bind(torrent_form.video_resolution_other_x.as_deref())
            .bind(torrent_form.video_resolution_other_y.as_deref())
            .bind(&*torrent_form.container.to_lowercase())
            .bind(
                torrent_form
                    .languages
                    .0
                    .split(',')
                    .filter(|f| !f.is_empty())
                    .map(|f| f.trim())
                    .collect::<Vec<&str>>(),
            )
            .bind(info_hash.as_ref())
            .bind(info.to_bytes())
            .bind(
                torrent_form
                    .extras
                    .split(',')
                    .filter(|f| !f.is_empty())
                    .map(|f| f.trim())
                    .collect::<Vec<&str>>(),
            )
            .bind(torrent_form.extra_text.as_deref().and_then(|s| {
                if s.trim().is_empty() {
                    None
                } else {
                    Some(s.trim().to_string())
                }
            }))
            .bind(bonus_points_snatch_cost)
            .fetch_one(&mut *tx)
            .await
            .map_err(Error::CouldNotCreateTorrent)?;

        let title_group_info = sqlx::query_as!(
            TitleGroupInfoLite,
            r#"
                SELECT
                    title_groups.id,
                    title_groups.name,
                    series.name AS "series_name?"
                FROM edition_groups
                JOIN title_groups ON edition_groups.title_group_id = title_groups.id
                LEFT JOIN series ON series.id = title_groups.series_id
                WHERE edition_groups.id = $1
            "#,
            torrent_form.edition_group_id.0
        )
        .fetch_one(&mut *tx)
        .await?;

        let user_ids = Self::notify_users_title_group_torrents(
            &mut tx,
            title_group_info.id,
            uploaded_torrent.id,
            user_id,
        )
        .await
        .unwrap_or_default();

        // Update torrents_amount for all affiliated artists of this title group
        sqlx::query!(
            r#"
            UPDATE artists
            SET torrents_amount = torrents_amount + 1
            WHERE id IN (
                SELECT DISTINCT artist_id
                FROM affiliated_artists
                WHERE title_group_id = $1
            )
            "#,
            title_group_info.id
        )
        .execute(&mut *tx)
        .await?;

        // Increment user's torrents counter and add bonus points for uploading
        sqlx::query!(
            r#"
            UPDATE users
            SET torrents = torrents + 1,
                bonus_points = bonus_points + $2
            WHERE id = $1
            "#,
            user_id,
            bonus_points_given_on_upload
        )
        .execute(&mut *tx)
        .await?;

        if bonus_points_given_on_upload > 0 {
            let details = format_title_group_name(
                title_group_info.series_name.as_deref(),
                &title_group_info.name,
            );
            Self::log_bonus_points_change_tx(
                &mut tx,
                user_id,
                BonusPointsLogAction::TorrentUploadReward,
                bonus_points_given_on_upload,
                Some(&details),
                Some(uploaded_torrent.id as i64),
            )
            .await?;
        }

        tx.commit().await?;

        if !user_ids.is_empty() {
            let _ = notification_sender.send(NotificationEvent::TitleGroupTorrent { user_ids });
        }

        Ok(uploaded_torrent)
    }

    pub async fn find_torrent(&self, torrent_id: i32) -> Result<Torrent> {
        let torrent = sqlx::query_as!(
            Torrent,
            r#"
            SELECT
                id, info_hash as "info_hash: InfoHash", upload_factor, download_factor, seeders, leechers,
                times_completed, grabbed, edition_group_id, created_at, updated_at,
                created_by_id,
                deleted_at AS "deleted_at!: _",
                deleted_by_id AS "deleted_by_id!: _",
                extras AS "extras!: _",
                languages AS "languages!: _",
                release_name, release_group, description, file_amount_per_type,
                uploaded_as_anonymous, upload_method, file_list, mediainfo, trumpable, staff_checked,
                container, size, duration,
                audio_codec AS "audio_codec: _",
                audio_bitrate,
                audio_bitrate_sampling AS "audio_bitrate_sampling: _",
                audio_channels AS "audio_channels: _",
                video_codec AS "video_codec: _",
                features AS "features!: _",
                subtitle_languages AS "subtitle_languages!: _",
                video_resolution AS "video_resolution!: _",
                video_resolution_other_x,
                video_resolution_other_y,
                extra_text,
                bonus_points_snatch_cost
            FROM torrents
            WHERE id = $1 AND deleted_at is NULL
            "#,
            torrent_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::TorrentNotFound)?;

        Ok(torrent)
    }

    /// Updates a torrent. When `trumpable_notification` is given, a conversation holding it is
    /// sent to the uploader of the torrent by its sender, in the same transaction as the update.
    pub async fn update_torrent(
        &self,
        edited_torrent: &EditedTorrent,
        torrent_id: i32,
        trumpable_notification: Option<&TrumpableNotification>,
    ) -> Result<Torrent> {
        let mut transaction = self.pool.begin().await?;

        let updated_torrent = sqlx::query_as!(
            Torrent,
            r#"
            UPDATE torrents
            SET
                release_name = $2,
                release_group = $3,
                description = $4,
                uploaded_as_anonymous = $5,
                mediainfo = $6,
                container = $7,
                duration = $8,
                audio_codec = $9,
                audio_bitrate = $10,
                audio_bitrate_sampling = $11,
                audio_channels = $12,
                video_codec = $13,
                features = $14,
                subtitle_languages = $15,
                video_resolution = $16,
                video_resolution_other_x = $17,
                video_resolution_other_y = $18,
                languages = $19,
                extras = $20,
                trumpable = $21,
                extra_text = $22,
                bonus_points_snatch_cost = $23,
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING
                id, info_hash as "info_hash: InfoHash", upload_factor, download_factor, seeders, leechers,
                times_completed, grabbed, edition_group_id, created_at, updated_at,
                created_by_id,
                deleted_at AS "deleted_at!: _",
                deleted_by_id AS "deleted_by_id!: _",
                extras AS "extras!: _",
                languages AS "languages!: _",
                release_name, release_group, description, file_amount_per_type,
                uploaded_as_anonymous, upload_method, file_list, mediainfo, trumpable, staff_checked,
                container, size, duration,
                audio_codec AS "audio_codec: _",
                audio_bitrate,
                audio_bitrate_sampling AS "audio_bitrate_sampling: _",
                audio_channels AS "audio_channels: _",
                video_codec AS "video_codec: _",
                features AS "features!: _",
                subtitle_languages AS "subtitle_languages!: _",
                video_resolution AS "video_resolution!: _",
                video_resolution_other_x,
                video_resolution_other_y,
                extra_text,
                bonus_points_snatch_cost
            "#,
            torrent_id,
            edited_torrent.release_name,
            edited_torrent.release_group,
            edited_torrent.description,
            edited_torrent.uploaded_as_anonymous,
            edited_torrent.mediainfo,
            edited_torrent.container,
            edited_torrent.duration,
            edited_torrent.audio_codec as _,
            edited_torrent.audio_bitrate,
            edited_torrent.audio_bitrate_sampling as _,
            edited_torrent.audio_channels as _,
            edited_torrent.video_codec as _,
            edited_torrent.features as _,
            edited_torrent.subtitle_languages as _,
            edited_torrent.video_resolution as _,
            edited_torrent.video_resolution_other_x,
            edited_torrent.video_resolution_other_y,
            edited_torrent.languages as _,
            edited_torrent.extras as _,
            edited_torrent.trumpable as _,
            edited_torrent.extra_text,
            edited_torrent.bonus_points_snatch_cost
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(|e| Error::ErrorWhileUpdatingTorrent(e.to_string()))?;

        if let Some(notification) = trumpable_notification {
            Self::send_batch_messages_tx(
                &mut transaction,
                notification.sender_id,
                &[updated_torrent.created_by_id],
                TORRENT_MARKED_TRUMPABLE_SUBJECT,
                &notification.content,
                true,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(updated_torrent)
    }

    pub async fn move_torrent_to_edition_group(
        &self,
        torrent_id: i32,
        target_edition_group_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE torrents
            SET edition_group_id = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            torrent_id,
            target_edition_group_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::ErrorWhileUpdatingTorrent(e.to_string()))?;

        Ok(())
    }

    pub async fn get_torrent(
        &self,
        user_id: i32,
        torrent_id: i32,
        tracker_name: &str,
        frontend_url: &str,
        tracker_url: &str,
        torrent_source_tag: Option<&str>,
    ) -> Result<GetTorrentResult> {
        let user = self.find_user_with_id(user_id).await?;
        self.get_torrent_with_passkey(
            user_id,
            &user.passkey,
            torrent_id,
            tracker_name,
            frontend_url,
            tracker_url,
            torrent_source_tag,
        )
        .await
    }

    /// Builds a single `.torrent` file for the given user, taking an already-resolved `passkey`
    /// instead of looking the user up again. Callers that build many torrents in a row (the bulk
    /// archive download) resolve the passkey once and reuse it, avoiding one user lookup per
    /// torrent.
    pub async fn get_torrent_with_passkey(
        &self,
        user_id: i32,
        passkey: &str,
        torrent_id: i32,
        tracker_name: &str,
        frontend_url: &str,
        tracker_url: &str,
        torrent_source_tag: Option<&str>,
    ) -> Result<GetTorrentResult> {
        let mut tx = self.pool.begin().await?;

        let torrent = sqlx::query!(
            r#"
            UPDATE torrents
            SET grabbed = grabbed + 1
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING
                info_dict,
                EXTRACT(EPOCH FROM created_at)::BIGINT AS "created_at_secs!",
                release_name;
            "#,
            torrent_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => Error::TorrentFileInvalid,
            other => other.into(),
        })?;

        let info = Info::from_bytes(torrent.info_dict).map_err(|_| Error::TorrentFileInvalid)?;

        let announce_url = get_announce_url(passkey, tracker_url);

        let frontend_url = format!("{frontend_url}torrent/{torrent_id}");

        let metainfo = MetainfoBuilder::new()
            .set_main_tracker(Some(&announce_url))
            .set_creation_date(Some(torrent.created_at_secs))
            .set_comment(Some(&frontend_url))
            .set_created_by(Some(tracker_name))
            .with_normalized_info_fields(&info, torrent_source_tag)
            .build(1, &info, |_| {})
            .map_err(|_| Error::TorrentFileInvalid)?;

        let _ = sqlx::query!(
            r#"
                INSERT INTO torrent_activities(torrent_id, user_id, grabbed_at)
                VALUES ($1, $2, NOW())
                ON CONFLICT (torrent_id, user_id) DO NOTHING;
            "#,
            torrent_id,
            user_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| Error::InvalidUserIdOrTorrentId);

        tx.commit().await?;

        Ok(GetTorrentResult {
            title: torrent.release_name,
            file_contents: metainfo,
        })
    }

    /// Returns the ids of every torrent uploaded by the given user, most recent first.
    /// Deleted torrents are left out. Meant to feed a bulk .torrent archive download.
    pub async fn get_uploaded_torrent_ids(&self, user_id: i32) -> Result<Vec<i32>> {
        sqlx::query_scalar!(
            r#"
            SELECT id
            FROM torrents
            WHERE created_by_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC;
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotGetUploadedTorrents)
    }

    /// Returns the ids of every torrent the given user has snatched (completed), most recent
    /// snatch first. Deleted torrents are left out. Meant to feed a bulk .torrent archive download.
    pub async fn get_snatched_torrent_ids(&self, user_id: i32) -> Result<Vec<i32>> {
        sqlx::query_scalar!(
            r#"
            SELECT torrents.id
            FROM torrents
            INNER JOIN torrent_activities
                ON torrent_activities.torrent_id = torrents.id
            WHERE torrent_activities.user_id = $1
                AND torrent_activities.completed_at IS NOT NULL
                AND torrents.deleted_at IS NULL
            ORDER BY torrent_activities.completed_at DESC;
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotGetSnatchedTorrents)
    }

    /// Recomputes the stored `info_hash` of every torrent so that it includes
    /// the configured source tag, matching the .torrent file served on download.
    ///
    /// This is a one-time maintenance step for trackers that already had torrents
    /// before `TORRENT_SOURCE_TAG` was set: those torrents were hashed without the
    /// source tag, so clients announcing the freshly downloaded (source-tagged)
    /// .torrent got `InfoHashNotFound`. The tracker rebuilds its
    /// infohash lookup table from this column on startup, so it must be restarted
    /// after running this. Idempotent: rows already matching are left untouched.
    ///
    /// Returns the number of torrents whose hash was updated.
    ///
    /// Torrents are processed in batches via keyset pagination on the primary key,
    /// so memory stays bounded even with lots of rows. The whole rehash runs in a
    /// single transaction, so a failure partway through rolls everything back and
    /// the operation can simply be re-run from a clean state.
    pub async fn rehash_torrents_with_source_tag(
        &self,
        torrent_source_tag: Option<&str>,
    ) -> Result<u64> {
        const BATCH_SIZE: i64 = 100;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::ErrorWhileUpdatingTorrent(e.to_string()))?;

        let mut last_seen_id: i32 = 0;
        let mut updated_count: u64 = 0;

        loop {
            let batch = sqlx::query!(
                "SELECT id, info_dict FROM torrents WHERE id > $1 ORDER BY id ASC LIMIT $2",
                last_seen_id,
                BATCH_SIZE
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| Error::ErrorWhileUpdatingTorrent(e.to_string()))?;

            if batch.is_empty() {
                break;
            }

            for torrent in batch {
                last_seen_id = torrent.id;

                let info =
                    Info::from_bytes(torrent.info_dict).map_err(|_| Error::TorrentFileInvalid)?;
                let info_hash = compute_torrent_info_hash(&info, torrent_source_tag)?;

                let result = sqlx::query!(
                    "UPDATE torrents SET info_hash = $1 WHERE id = $2 AND info_hash IS DISTINCT FROM $1",
                    info_hash.as_ref(),
                    torrent.id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::ErrorWhileUpdatingTorrent(e.to_string()))?;

                updated_count += result.rows_affected();
            }
        }

        tx.commit()
            .await
            .map_err(|e| Error::ErrorWhileUpdatingTorrent(e.to_string()))?;

        Ok(updated_count)
    }

    pub async fn search_torrents(
        &self,
        form: &TorrentSearch,
        requesting_user_id: Option<i32>,
        can_see_anonymous_uploads: bool,
    ) -> Result<PaginatedResults<TitleGroupHierarchyLite>> {
        let (name_filter, external_link_filter) = match &form.title_group_name {
            Some(s) => {
                let input = s.trim();
                if input.is_empty() {
                    (None, None)
                } else if looks_like_url(input) {
                    (None, Some(input.trim_end_matches('/').to_string()))
                } else {
                    (Some(input.to_string()), None)
                }
            }
            None => (None, None),
        };

        let tag_filter_jsonb: Option<serde_json::Value> = match &form.title_group_tags {
            Some(s) => crate::utils::tag_expression::parse_tag_expression(s)
                .map_err(Error::InvalidTagExpression)?,
            None => None,
        };

        let limit = form.page_size;
        let offset = (form.page - 1) * form.page_size;

        // we apply filters on 3 tables: title_groups, edition_groups, and torrents

        // first: get title groups that have editions and torrents (and the title groups themselves)
        // matching the filters on the 3 tables right away, thanks to the materialized view

        let title_groups = sqlx::query_as!(
            TitleGroupHierarchyLite,
            r#"
             SELECT title_group_id AS "id!", title_group_name AS "name!", title_group_covers AS "covers!",
             title_group_category AS "category!: _", title_group_content_type AS "content_type!: _", title_group_tag_names AS "tags!",
             title_group_original_release_date AS "original_release_date",
             title_group_original_release_date_only_year_known AS "original_release_date_only_year_known!",
             title_group_platform AS "platform!: _",
             '[]'::jsonb AS "edition_groups!: _",
             '[]'::jsonb AS "affiliated_artists!: _",
             CASE
                WHEN title_group_series_id IS NOT NULL THEN jsonb_build_object('id', title_group_series_id, 'name', title_group_series_name)
                ELSE NULL
             END AS "series: _"

             FROM title_group_hierarchy_lite tgh

             WHERE ($4::BOOLEAN IS NULL OR tgh.torrent_staff_checked = $4)
             AND ($5::BOOLEAN IS NULL OR tgh.torrent_reported = $5)
             AND ($24::BOOLEAN IS NULL OR (NULLIF(tgh.torrent_trumpable, '') IS NOT NULL) = $24)
             AND (
                $7::INT IS NULL OR
                -- don't return torrents created as anonymous
                -- unless the requesting user is the uploader or is allowed to see them
                (tgh.torrent_created_by_id = $7 AND (
                   tgh.torrent_created_by_id = $8 OR
                   $23::BOOLEAN OR
                   NOT tgh.torrent_uploaded_as_anonymous)
                )
            )
            AND (
                $9::BIGINT IS NULL OR
                EXISTS (SELECT 1 FROM affiliated_artists aa WHERE aa.title_group_id = tgh.title_group_id AND aa.artist_id = $9)
            )
            -- name filter (partial match) or external link match or series name match
            AND (
                $10::TEXT IS NULL OR
                tgh.title_group_name ILIKE '%' || $10 || '%' ESCAPE '\' OR
                tgh.title_group_series_name ILIKE '%' || $10 || '%' ESCAPE '\' OR
                EXISTS (SELECT 1 FROM unnest(tgh.title_group_name_aliases) alias WHERE alias ILIKE '%' || $10 || '%')
            )
            AND (
                $11::TEXT IS NULL
                OR EXISTS (
                    SELECT 1 FROM unnest(tgh.title_group_external_links) link
                    WHERE rtrim(link, '/') = $11
                )
            )
            AND ($12::BOOLEAN IS TRUE OR tgh.torrent_id IS NOT NULL)
            AND ($13::BIGINT IS NULL OR tgh.title_group_series_id = $13)
            AND (
                $14::INT IS NULL OR
                EXISTS (SELECT 1 FROM collage_entry ce WHERE ce.title_group_id = tgh.title_group_id AND ce.collage_id = $14)
            )
            AND (CARDINALITY($15::content_type_enum[]) = 0 OR tgh.title_group_content_type = ANY($15))
            AND (CARDINALITY($16::title_group_category_enum[]) = 0 OR tgh.title_group_category = ANY($16))
            AND (CARDINALITY($17::source_enum[]) = 0 OR tgh.edition_group_source = ANY($17))
            AND (CARDINALITY($18::video_resolution_enum[]) = 0 OR tgh.torrent_video_resolution = ANY($18))
            AND (CARDINALITY($19::language_enum[]) = 0 OR tgh.torrent_languages && $19)
            AND (
                $20::INT IS NULL OR
                EXISTS (
                    SELECT 1 FROM torrent_activities ta
                    WHERE ta.torrent_id = tgh.torrent_id
                    AND ta.user_id = $20
                    AND ta.completed_at IS NOT NULL
                )
            )
            AND (
                $21::JSONB IS NULL OR
                EXISTS (
                    SELECT 1 FROM jsonb_array_elements($21) AS clause
                    WHERE COALESCE(ARRAY(SELECT jsonb_array_elements_text(clause->'include'))::varchar[], '{}') <@ title_group_tag_names
                    AND NOT title_group_tag_names && COALESCE(ARRAY(SELECT jsonb_array_elements_text(clause->'exclude'))::varchar[], '{}')
                )
            )
            AND (
                $22::BIGINT IS NULL OR
                EXISTS (
                    SELECT 1 FROM title_group_bookmarks tgb
                    WHERE tgb.title_group_id = tgh.title_group_id
                    AND tgb.user_id = $22
                )
            )

            GROUP BY title_group_id, title_group_name, title_group_covers, title_group_category,
            title_group_content_type, title_group_tag_names, title_group_original_release_date,
            title_group_original_release_date_only_year_known, title_group_platform,
            tgh.title_group_series_id, tgh.title_group_series_name

            ORDER BY
                CASE WHEN $1 = 'title_group_original_release_date' AND $6 = 'asc' THEN title_group_original_release_date END ASC NULLS LAST,
                CASE WHEN $1 = 'title_group_original_release_date' AND $6 = 'desc' THEN title_group_original_release_date END DESC NULLS LAST,
                CASE WHEN $1 = 'torrent_size' AND $6 = 'asc' THEN MIN(torrent_size) END ASC,
                CASE WHEN $1 = 'torrent_size' AND $6 = 'desc' THEN MAX(torrent_size) END DESC,
                CASE WHEN $1 = 'torrent_created_at' AND $6 = 'asc' THEN MIN(torrent_created_at) END ASC,
                CASE WHEN $1 = 'torrent_created_at' AND $6 = 'desc' THEN MAX(torrent_created_at) END DESC,
                CASE WHEN $1 = 'torrent_seeders' AND $6 = 'asc' THEN MIN(torrent_seeders) END ASC,
                CASE WHEN $1 = 'torrent_seeders' AND $6 = 'desc' THEN MAX(torrent_seeders) END DESC,
                CASE WHEN $1 = 'torrent_leechers' AND $6 = 'asc' THEN MIN(torrent_leechers) END ASC,
                CASE WHEN $1 = 'torrent_leechers' AND $6 = 'desc' THEN MAX(torrent_leechers) END DESC,
                CASE WHEN $1 = 'torrent_snatched' AND $6 = 'asc' THEN MIN(torrent_times_completed) END ASC,
                CASE WHEN $1 = 'torrent_snatched' AND $6 = 'desc' THEN MAX(torrent_times_completed) END DESC,
                CASE WHEN $1 = 'torrent_bonus_points_snatch_cost' AND $6 = 'asc' THEN
                    MIN((SELECT t.bonus_points_snatch_cost FROM torrents t WHERE t.id = tgh.torrent_id))
                END ASC NULLS LAST,
                CASE WHEN $1 = 'torrent_bonus_points_snatch_cost' AND $6 = 'desc' THEN
                    MAX((SELECT t.bonus_points_snatch_cost FROM torrents t WHERE t.id = tgh.torrent_id))
                END DESC NULLS LAST,
                CASE WHEN $1 = 'torrent_snatched_at' AND $6 = 'asc' THEN
                    MIN((SELECT ta.completed_at FROM torrent_activities ta WHERE ta.torrent_id = tgh.torrent_id AND ta.user_id = $20))
                END ASC NULLS LAST,
                CASE WHEN $1 = 'torrent_snatched_at' AND $6 = 'desc' THEN
                    MAX((SELECT ta.completed_at FROM torrent_activities ta WHERE ta.torrent_id = tgh.torrent_id AND ta.user_id = $20))
                END DESC NULLS LAST,
                title_group_original_release_date ASC,
                title_group_name ASC

            LIMIT $2 OFFSET $3
            "#,
            form.order_by_column.to_string(),
            limit,
            offset,
            form.torrent_staff_checked,
            form.torrent_reported,
            form.order_by_direction.to_string(),
            form.torrent_created_by_id,
            requesting_user_id,
            form.artist_id,
            name_filter,
            external_link_filter,
            form.title_group_include_empty_groups,
            form.series_id,
            form.collage_id,
            form.title_group_content_type.as_slice() as &[ContentType],
            form.title_group_category.as_slice() as &[TitleGroupCategory],
            form.edition_group_source.as_slice() as &[Source],
            form.torrent_video_resolution.as_slice() as &[VideoResolution],
            form.torrent_language.as_slice() as &[Language],
            form.torrent_snatched_by_id,
            tag_filter_jsonb.clone() as Option<serde_json::Value>,
            form.user_id_bookmarks,
            can_see_anonymous_uploads,
            form.torrent_trumpable
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| Error::ErrorSearchingForTorrents(error.to_string()))?;

        // amount of results for pagination
        let total_title_groups_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT title_group_id)
            FROM title_group_hierarchy_lite tgh
            WHERE ($1::BOOLEAN IS NULL OR tgh.torrent_staff_checked = $1)
              AND ($2::BOOLEAN IS NULL OR tgh.torrent_reported = $2)
              AND ($20::BOOLEAN IS NULL OR (NULLIF(tgh.torrent_trumpable, '') IS NOT NULL) = $20)
              AND (
                 $3::INT IS NULL OR
                 -- don't return torrents created as anonymous
                 -- unless the requesting user is the uploader or is allowed to see them
                 (tgh.torrent_created_by_id = $3 AND (
                    tgh.torrent_created_by_id = $4 OR
                    $19::BOOLEAN OR
                    NOT tgh.torrent_uploaded_as_anonymous)
                 )
             )

            AND (
                $5::TEXT IS NULL OR
                    tgh.title_group_name ILIKE '%' || $5 || '%' ESCAPE '\' OR
                    tgh.title_group_series_name ILIKE '%' || $5 || '%' ESCAPE '\' OR
                    EXISTS (SELECT 1 FROM unnest(tgh.title_group_name_aliases) alias WHERE alias ILIKE '%' || $5 || '%')
            )
            AND (
                $6::TEXT IS NULL
                OR EXISTS (
                    SELECT 1 FROM unnest(tgh.title_group_external_links) link
                    WHERE rtrim(link, '/') = $6
                )
            )
            AND ($7::BOOLEAN IS TRUE OR tgh.torrent_id IS NOT NULL)
            AND (
                $8::INT IS NULL OR
                EXISTS (SELECT 1 FROM collage_entry ce WHERE ce.title_group_id = tgh.title_group_id AND ce.collage_id = $8)
            )
            AND (
                $18::BIGINT IS NULL OR
                EXISTS (SELECT 1 FROM affiliated_artists aa WHERE aa.title_group_id = tgh.title_group_id AND aa.artist_id = $18)
            )
            AND (CARDINALITY($9::content_type_enum[]) = 0 OR tgh.title_group_content_type = ANY($9))
            AND (CARDINALITY($10::title_group_category_enum[]) = 0 OR tgh.title_group_category = ANY($10))
            AND (CARDINALITY($11::source_enum[]) = 0 OR tgh.edition_group_source = ANY($11))
            AND (CARDINALITY($12::video_resolution_enum[]) = 0 OR tgh.torrent_video_resolution = ANY($12))
            AND (CARDINALITY($13::language_enum[]) = 0 OR tgh.torrent_languages && $13)
            AND ($14::BIGINT IS NULL OR tgh.title_group_series_id = $14)
            AND (
                $15::INT IS NULL OR
                EXISTS (
                    SELECT 1 FROM torrent_activities ta
                    WHERE ta.torrent_id = tgh.torrent_id
                    AND ta.user_id = $15
                    AND ta.completed_at IS NOT NULL
                )
            )
            AND (
                $16::JSONB IS NULL OR
                EXISTS (
                    SELECT 1 FROM jsonb_array_elements($16) AS clause
                    WHERE COALESCE(ARRAY(SELECT jsonb_array_elements_text(clause->'include'))::varchar[], '{}') <@ title_group_tag_names
                    AND NOT title_group_tag_names && COALESCE(ARRAY(SELECT jsonb_array_elements_text(clause->'exclude'))::varchar[], '{}')
                )
            )
            AND (
                $17::BIGINT IS NULL OR
                EXISTS (
                    SELECT 1 FROM title_group_bookmarks tgb
                    WHERE tgb.title_group_id = tgh.title_group_id
                    AND tgb.user_id = $17
                )
            )
            "#,
            form.torrent_staff_checked,
            form.torrent_reported,
            form.torrent_created_by_id,
            requesting_user_id,
            name_filter,
            external_link_filter,
            form.title_group_include_empty_groups,
            form.collage_id,
            form.title_group_content_type.as_slice() as &[ContentType],
            form.title_group_category.as_slice() as &[TitleGroupCategory],
            form.edition_group_source.as_slice() as &[Source],
            form.torrent_video_resolution.as_slice() as &[VideoResolution],
            form.torrent_language.as_slice() as &[Language],
            form.series_id,
            form.torrent_snatched_by_id,
            tag_filter_jsonb as Option<serde_json::Value>,
            form.user_id_bookmarks,
            form.artist_id,
            can_see_anonymous_uploads,
            form.torrent_trumpable
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| Error::ErrorSearchingForTorrents(error.to_string()))?
        .unwrap_or(Some(0));

        // second: get the edition groups that match the edition group filters, that are within the title groups
        // from the previous step

        let title_group_ids: Vec<i32> = title_groups.iter().map(|t| t.id).collect();

        let edition_groups = sqlx::query_as!(
            EditionGroupHierarchyLite,
            r#"
            SELECT
                id,
                title_group_id,
                name,
                release_date,
                release_date_only_year_known,
                distributor,
                covers,
                source AS "source: _",
                additional_information AS "additional_information: _",
                '[]'::jsonb AS "torrents!: _"
            FROM edition_groups
            WHERE title_group_id = ANY($1)
            AND (CARDINALITY($2::source_enum[]) = 0 OR source = ANY($2))
            "#,
            &title_group_ids,
            form.edition_group_source.as_slice() as &[Source]
        )
        .fetch_all(&self.pool)
        .await?;

        let mut grouped_editions: HashMap<i32, Vec<EditionGroupHierarchyLite>> = HashMap::new();

        for eg in edition_groups {
            grouped_editions.entry(eg.title_group_id).or_default().push(
                EditionGroupHierarchyLite {
                    torrents: Json(Vec::new()),
                    ..eg
                },
            );
        }

        let title_groups: Vec<TitleGroupHierarchyLite> = title_groups
            .into_iter()
            .map(|mut tg| {
                tg.edition_groups = Json(grouped_editions.remove(&tg.id).unwrap_or_default());
                tg
            })
            .collect();

        // third: get the torrents that match the torrent filters, and are in the edition groups
        // from the previous step

        let edition_group_ids: Vec<i32> = title_groups
            .iter()
            .flat_map(|tg| tg.edition_groups.0.iter().map(|eg| eg.id))
            .collect();

        let torrents = sqlx::query_as!(
            TorrentHierarchyLite,
            r#"
            SELECT
                tar.id AS "id!",
                tar.upload_factor AS "upload_factor!",
                tar.download_factor AS "download_factor!",
                tar.seeders AS "seeders!",
                tar.leechers AS "leechers!",
                tar.times_completed AS "times_completed!",
                tar.grabbed AS "grabbed!",
                tar.edition_group_id AS "edition_group_id!",
                tar.created_at AS "created_at!: _",
                CASE
                    -- the permission to see the anonymous uploads only reveals the uploader
                    -- searched by the requesting user, not everyone in the results
                    WHEN tar.uploaded_as_anonymous AND tar.created_by_id != $5
                        AND NOT ($9::BOOLEAN AND tar.created_by_id IS NOT DISTINCT FROM $2) THEN
                        NULL
                    ELSE
                        ROW(u.id, u.username, u.warned, u.banned)
                END AS "created_by: UserLite",
                tar.uploaded_as_anonymous AS "uploaded_as_anonymous!",
                tar.release_name,
                tar.release_group,
                tar.trumpable,
                tar.staff_checked AS "staff_checked!",
                COALESCE(tar.languages, '{}') AS "languages!: _",
                tar.container AS "container!",
                tar.size AS "size!",
                tar.duration,
                tar.audio_codec AS "audio_codec: _",
                tar.audio_bitrate,
                tar.audio_bitrate_sampling AS "audio_bitrate_sampling: _",
                tar.audio_channels AS "audio_channels: _",
                tar.video_codec AS "video_codec: _",
                tar.features AS "features!: _",
                COALESCE(tar.subtitle_languages, '{}') AS "subtitle_languages!: _",
                tar.video_resolution AS "video_resolution: _",
                tar.video_resolution_other_x,
                tar.video_resolution_other_y,
                tar.extra_text,
                tar.reports AS "reports!: _",
                COALESCE(tar.extras, '{}') AS "extras!: _",
                CASE
                    WHEN EXISTS (
                        SELECT 1 FROM peers
                        WHERE torrent_id = tar.id
                        AND user_id = $5
                        AND active = true
                        AND seeder = true
                    ) THEN 'seeding'
                    WHEN EXISTS (
                        SELECT 1 FROM peers
                        WHERE torrent_id = tar.id
                        AND user_id = $5
                        AND active = true
                        AND seeder = false
                    ) THEN 'leeching'
                    WHEN EXISTS (
                        SELECT 1 FROM torrent_activities
                        WHERE torrent_id = tar.id
                        AND user_id = $5
                        AND completed_at IS NOT NULL
                    ) THEN 'snatched'
                    WHEN EXISTS (
                        SELECT 1 FROM torrent_activities
                        WHERE torrent_id = tar.id
                        AND user_id = $5
                        AND grabbed_at IS NOT NULL
                    ) AND NOT EXISTS (
                        SELECT 1 FROM peers
                        WHERE torrent_id = tar.id
                        AND user_id = $5
                        AND active = true
                    ) THEN 'grabbed'
                    ELSE NULL
                END AS "peer_status: _",
                tar.bonus_points_snatch_cost AS "bonus_points_snatch_cost!",
                tar.upload_method AS "upload_method!"
            FROM torrents_and_reports tar
            JOIN users u ON tar.created_by_id = u.id
            WHERE tar.edition_group_id = ANY($1)

            AND ($3::BOOLEAN IS NULL OR tar.staff_checked = $3)
            AND ($4::BOOLEAN IS NULL OR tar.reported = $4)
            AND ($10::BOOLEAN IS NULL OR (NULLIF(tar.trumpable, '') IS NOT NULL) = $10)
            AND (
               $2::INT IS NULL OR
               -- don't return torrents created as anonymous
               -- unless the requesting user is the uploader or is allowed to see them
               (tar.created_by_id = $2 AND (
                  tar.created_by_id = $5 OR
                  $9::BOOLEAN OR
                  NOT tar.uploaded_as_anonymous)
               )
            )
            AND (CARDINALITY($6::video_resolution_enum[]) = 0 OR tar.video_resolution = ANY($6))
            AND (CARDINALITY($7::language_enum[]) = 0 OR tar.languages && $7)
            AND (
                $8::INT IS NULL OR
                EXISTS (
                    SELECT 1 FROM torrent_activities ta
                    WHERE ta.torrent_id = tar.id
                    AND ta.user_id = $8
                    AND ta.grabbed_at IS NOT NULL
                )
            )

            ORDER BY size DESC
            "#,
            &edition_group_ids,
            form.torrent_created_by_id,
            form.torrent_staff_checked,
            form.torrent_reported,
            requesting_user_id,
            form.torrent_video_resolution.as_slice() as &[VideoResolution],
            form.torrent_language.as_slice() as &[Language],
            form.torrent_snatched_by_id,
            can_see_anonymous_uploads,
            form.torrent_trumpable
        )
        .fetch_all(&self.pool)
        .await?;

        let mut grouped_torrents: HashMap<i32, Vec<TorrentHierarchyLite>> = HashMap::new();

        for t in torrents {
            grouped_torrents
                .entry(t.edition_group_id)
                .or_default()
                .push(t);
        }

        let mut grouped_artists = self
            .find_affiliated_artists_lite_by_title_group_ids(&title_group_ids)
            .await?;

        let title_groups = title_groups
            .into_iter()
            .map(|mut tg| {
                let edition_groups_with_torrents: Vec<EditionGroupHierarchyLite> = tg
                    .edition_groups
                    .0
                    .into_iter()
                    .map(|mut eg| {
                        eg.torrents = Json(grouped_torrents.remove(&eg.id).unwrap_or_default());
                        eg
                    })
                    .collect();

                tg.edition_groups = Json(edition_groups_with_torrents);
                tg.affiliated_artists = Json(grouped_artists.remove(&tg.id).unwrap_or_default());
                tg
            })
            .collect();

        Ok(PaginatedResults {
            results: title_groups,
            page: form.page as u32,
            page_size: form.page_size as u32,
            total_items: total_title_groups_count.unwrap_or(0),
        })
    }

    #[expect(
        clippy::unused_async,
        reason = "stub until the top torrents query is restored"
    )]
    pub async fn find_top_torrents(&self, _period: &str, _amount: i64) -> Result<Value> {
        Ok(Value::Array(vec![]))
        // let search_results = sqlx::query!(
        //     r#"
        //     WITH title_group_search AS (
        //         ---------- This is the part that selects the top torrents
        //         SELECT DISTINCT ON (tg.id) tg.id AS title_group_id
        //         FROM torrents t
        //         JOIN torrent_activities st ON t.id = st.torrent_id
        //         JOIN edition_groups eg ON t.edition_group_id = eg.id
        //         JOIN title_groups tg ON eg.title_group_id = tg.id
        //         WHERE CASE
        //             WHEN $1 = 'all time' THEN TRUE
        //             ELSE t.created_at >= NOW() - CAST($1 AS INTERVAL)
        //         END AND t.deleted_at is NULL
        //         GROUP BY tg.id, tg.name
        //         ORDER BY tg.id, COUNT(st.torrent_id) DESC
        //         LIMIT $2
        //         ----------
        //     ),
        //     title_group_data AS (
        //         SELECT
        //             tgl.title_group_data AS lite_title_group -- 'affiliated_artists' is already inside tgl.title_group_data
        //         FROM get_title_groups_and_edition_group_and_torrents_lite tgl
        //         JOIN title_groups tg ON tgl.title_group_id = tg.id
        //         JOIN title_group_search tgs ON tg.id = tgs.title_group_id
        //     )
        //     SELECT jsonb_agg(lite_title_group) AS title_groups
        //     FROM title_group_data;
        //     "#,
        //     period,
        //     amount
        // )
        // .fetch_one(&self.pool)
        // .await
        // .map_err(|error| Error::ErrorSearchingForTorrents(error.to_string()))?;

        // Ok(serde_json::json!({"title_groups": search_results.title_groups}))
    }

    pub async fn remove_torrent(
        &self,
        torrent_to_delete: &TorrentToDelete,
        current_user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let title_group = sqlx::query!(
            r#"
            SELECT tg.id, tg.name
            FROM torrents t
            JOIN edition_groups eg ON t.edition_group_id = eg.id
            JOIN title_groups tg ON tg.id = eg.title_group_id
            WHERE t.id = $1
            "#,
            torrent_to_delete.id
        )
        .fetch_one(&mut *tx)
        .await?;
        let title_group_id = title_group.id;

        // Notifies every peer currently tied to the torrent (seeders and leechers)
        // as well as the original uploader, excluding the user performing the deletion.
        let notified_user_ids = ConnectionPool::record_torrent_deletion(
            &mut tx,
            torrent_to_delete.id,
            &title_group.name,
            torrent_to_delete.deletion_reason,
            torrent_to_delete.extra_information.as_deref(),
            torrent_to_delete.replacement_torrent_id,
            current_user_id,
        )
        .await?;

        sqlx::query!(
            r#"
            UPDATE torrents SET deleted_at = NOW(), deleted_by_id = $1 WHERE id = $2;
            "#,
            current_user_id,
            torrent_to_delete.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|error| Error::ErrorDeletingTorrent(error.to_string()))?;

        // Update torrents_amount for all affiliated artists of this title group
        sqlx::query!(
            r#"
            UPDATE artists
            SET torrents_amount = torrents_amount - 1
            WHERE id IN (
                SELECT DISTINCT artist_id
                FROM affiliated_artists
                WHERE title_group_id = $1
            )
            "#,
            title_group_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        if !notified_user_ids.is_empty() {
            let _ = notification_sender.send(NotificationEvent::TorrentDeletion {
                user_ids: notified_user_ids,
            });
        }

        Ok(())
    }

    pub async fn set_torrent_staff_checked(
        &self,
        torrent_id: i32,
        staff_checked: bool,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE torrents
            SET
                staff_checked = $2,
                updated_at = NOW()
            WHERE
                id = $1 AND deleted_at IS NULL
            "#,
            torrent_id,
            staff_checked
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_torrent_peers(
        &self,
        torrent_id: i32,
        requesting_user_id: i32,
    ) -> Result<Vec<PublicPeer>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                u.id AS user_id,
                u.username,
                u.warned,
                u.banned,
                p.ip,
                p.port,
                p.uploaded,
                p.downloaded,
                p.left,
                p.seeder,
                p.agent
            FROM peers p
            JOIN users u ON p.user_id = u.id
            WHERE p.torrent_id = $1 AND p.active = true
            ORDER BY p.seeder DESC, p.uploaded DESC
            "#,
            torrent_id
        )
        .fetch_all(&self.pool)
        .await?;

        let peers = rows
            .into_iter()
            .map(|row| {
                let is_own_peer = row.user_id == requesting_user_id;
                PublicPeer {
                    user: UserLite {
                        id: row.user_id,
                        username: row.username,
                        warned: row.warned,
                        banned: row.banned,
                    },
                    ip: if is_own_peer { Some(row.ip) } else { None },
                    port: if is_own_peer { Some(row.port) } else { None },
                    uploaded: row.uploaded,
                    downloaded: row.downloaded,
                    left: row.left,
                    seeder: row.seeder,
                    agent: row.agent,
                }
            })
            .collect();

        Ok(peers)
    }

    pub async fn get_torrent_activities_of_torrent(
        &self,
        torrent_id: i32,
        page: u32,
        page_size: u32,
        order_by_column: TorrentActivityOfTorrentOrderByColumn,
        order_by_direction: OrderByDirection,
    ) -> Result<PaginatedResults<TorrentActivityAndUser>> {
        let results = sqlx::query_as!(
            TorrentActivityAndUser,
            r#"
            SELECT
                ROW(u.id, u.username, u.warned, u.banned) AS "user!: UserLite",
                ta.grabbed_at, ta.completed_at, ta.first_seen_seeding_at, ta.last_seen_seeding_at,
                ta.total_seed_time, ta.bonus_points, ta.uploaded, ta.real_uploaded,
                ta.downloaded, ta.real_downloaded,
                peer_status.seeder AS "seeder!"
            FROM torrent_activities ta
            JOIN users u ON u.id = ta.user_id
            CROSS JOIN LATERAL (
                SELECT EXISTS (
                    SELECT 1 FROM peers p
                    WHERE p.torrent_id = ta.torrent_id
                      AND p.user_id = ta.user_id
                      AND p.seeder = true
                      AND p.active = true
                ) AS seeder
            ) peer_status
            WHERE ta.torrent_id = $1
            ORDER BY
                CASE WHEN $4 = 'username' AND $5 = 'asc' THEN u.username END ASC,
                CASE WHEN $4 = 'username' AND $5 = 'desc' THEN u.username END DESC,
                CASE WHEN $4 = 'seeder' AND $5 = 'asc' THEN peer_status.seeder END ASC,
                CASE WHEN $4 = 'seeder' AND $5 = 'desc' THEN peer_status.seeder END DESC,
                CASE WHEN $4 = 'uploaded' AND $5 = 'asc' THEN ta.uploaded END ASC,
                CASE WHEN $4 = 'uploaded' AND $5 = 'desc' THEN ta.uploaded END DESC,
                CASE WHEN $4 = 'real_uploaded' AND $5 = 'asc' THEN ta.real_uploaded END ASC,
                CASE WHEN $4 = 'real_uploaded' AND $5 = 'desc' THEN ta.real_uploaded END DESC,
                CASE WHEN $4 = 'downloaded' AND $5 = 'asc' THEN ta.downloaded END ASC,
                CASE WHEN $4 = 'downloaded' AND $5 = 'desc' THEN ta.downloaded END DESC,
                CASE WHEN $4 = 'real_downloaded' AND $5 = 'asc' THEN ta.real_downloaded END ASC,
                CASE WHEN $4 = 'real_downloaded' AND $5 = 'desc' THEN ta.real_downloaded END DESC,
                CASE WHEN $4 = 'total_seed_time' AND $5 = 'asc' THEN ta.total_seed_time END ASC,
                CASE WHEN $4 = 'total_seed_time' AND $5 = 'desc' THEN ta.total_seed_time END DESC,
                CASE WHEN $4 = 'bonus_points' AND $5 = 'asc' THEN ta.bonus_points END ASC,
                CASE WHEN $4 = 'bonus_points' AND $5 = 'desc' THEN ta.bonus_points END DESC,
                CASE WHEN $4 = 'grabbed_at' AND $5 = 'asc' THEN ta.grabbed_at END ASC NULLS LAST,
                CASE WHEN $4 = 'grabbed_at' AND $5 = 'desc' THEN ta.grabbed_at END DESC NULLS LAST,
                CASE WHEN $4 = 'completed_at' AND $5 = 'asc' THEN ta.completed_at END ASC NULLS LAST,
                CASE WHEN $4 = 'completed_at' AND $5 = 'desc' THEN ta.completed_at END DESC NULLS LAST,
                CASE WHEN $4 = 'first_seen_seeding_at' AND $5 = 'asc' THEN ta.first_seen_seeding_at END ASC NULLS LAST,
                CASE WHEN $4 = 'first_seen_seeding_at' AND $5 = 'desc' THEN ta.first_seen_seeding_at END DESC NULLS LAST,
                CASE WHEN $4 = 'last_seen_seeding_at' AND $5 = 'asc' THEN ta.last_seen_seeding_at END ASC NULLS LAST,
                CASE WHEN $4 = 'last_seen_seeding_at' AND $5 = 'desc' THEN ta.last_seen_seeding_at END DESC NULLS LAST,
                ta.id DESC
            LIMIT $2 OFFSET $3
            "#,
            torrent_id,
            page_size as i64,
            (page.saturating_sub(1) * page_size) as i64,
            order_by_column.to_string(),
            order_by_direction.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        let total_items = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM torrent_activities WHERE torrent_id = $1"#,
            torrent_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            page,
            page_size,
            total_items,
        })
    }

    pub async fn update_torrent_up_down_factors(
        &self,
        torrent_id: i32,
        upload_factor: i16,
        download_factor: i16,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE torrents
            SET
                upload_factor = $2,
                download_factor = $3
            WHERE
                id = $1
            "#,
            torrent_id,
            upload_factor,
            download_factor
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_torrent_activities(
        &self,
        user_id: i32,
        query: &GetTorrentActivitiesQuery,
        formula_sql: &str,
        ticks_per_day: i64,
    ) -> Result<PaginatedResults<TorrentActivityAndTitleGroup>> {
        let limit = query.page_size as i64;
        let offset = ((query.page - 1) * query.page_size) as i64;

        // Step 1: Get paginated torrent activity references
        #[derive(sqlx::FromRow)]
        struct TorrentActivityRef {
            torrent_id: i32,
            edition_group_id: i32,
            title_group_id: i32,
        }

        let order_by_expression = match query.order_by_column {
            TorrentActivityOrderByColumn::GrabbedAt => "ta.grabbed_at".to_string(),
            TorrentActivityOrderByColumn::TotalSeedTime => "ta.total_seed_time".to_string(),
            TorrentActivityOrderByColumn::Uploaded => "ta.uploaded".to_string(),
            TorrentActivityOrderByColumn::Downloaded => "ta.downloaded".to_string(),
            TorrentActivityOrderByColumn::TorrentSize => "tgh.torrent_size".to_string(),
            TorrentActivityOrderByColumn::TorrentSeeders => "tgh.torrent_seeders".to_string(),
            TorrentActivityOrderByColumn::BonusPoints => "ta.bonus_points".to_string(),
            TorrentActivityOrderByColumn::BonusPointsPerDay => format!(
                "COALESCE((SELECT ROUND({formula})::bigint FROM torrents t INNER JOIN peers p ON p.torrent_id = t.id AND p.user_id = ta.user_id WHERE t.id = ta.torrent_id AND t.staff_checked AND p.seeder = true AND p.active = true LIMIT 1), 0)",
                formula = formula_sql
            ),
        };

        let direction = match query.order_by_direction {
            OrderByDirection::Asc => "ASC",
            OrderByDirection::Desc => "DESC",
        };

        let nulls_clause = match query.order_by_column {
            TorrentActivityOrderByColumn::GrabbedAt => " NULLS LAST",
            _ => "",
        };

        let activity_refs_query = format!(
            r#"
            SELECT tgh.torrent_id,
                   tgh.edition_group_id,
                   tgh.title_group_id
            FROM title_group_hierarchy_lite tgh
            INNER JOIN torrent_activities ta ON ta.torrent_id = tgh.torrent_id
            WHERE ta.user_id = $1
              AND tgh.torrent_id IS NOT NULL
              AND ($4::BOOLEAN OR EXISTS (
                  SELECT 1 FROM peers p
                  WHERE p.torrent_id = tgh.torrent_id
                    AND p.user_id = $1
                    AND p.seeder = true
                    AND p.active = true
              ))
            ORDER BY {order_by} {direction}{nulls},
                     ta.grabbed_at DESC NULLS LAST
            LIMIT $2 OFFSET $3
            "#,
            order_by = order_by_expression,
            direction = direction,
            nulls = nulls_clause
        );

        let activity_refs: Vec<TorrentActivityRef> =
            sqlx::query_as(sqlx::AssertSqlSafe(activity_refs_query))
                .bind(user_id)
                .bind(limit)
                .bind(offset)
                .bind(query.include_unseeded_torrents)
                .fetch_all(&self.pool)
                .await
                .map_err(|error| Error::ErrorSearchingForTorrents(error.to_string()))?;

        // Step 2: Count total torrent activities
        let total_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM title_group_hierarchy_lite tgh
            INNER JOIN torrent_activities ta ON ta.torrent_id = tgh.torrent_id
            WHERE ta.user_id = $1
              AND tgh.torrent_id IS NOT NULL
              AND ($2::BOOLEAN OR EXISTS (
                  SELECT 1 FROM peers p
                  WHERE p.torrent_id = tgh.torrent_id
                    AND p.user_id = $1
                    AND p.seeder = true
                    AND p.active = true
              ))
            "#,
            user_id,
            query.include_unseeded_torrents
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| Error::ErrorSearchingForTorrents(error.to_string()))?
        .unwrap_or(Some(0));

        if activity_refs.is_empty() {
            return Ok(PaginatedResults {
                results: Vec::new(),
                page: query.page,
                page_size: query.page_size,
                total_items: total_count.unwrap_or(0),
            });
        }

        let torrent_ids: Vec<i32> = activity_refs.iter().map(|r| r.torrent_id).collect();
        let edition_group_ids: Vec<i32> = activity_refs
            .iter()
            .map(|r| r.edition_group_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let title_group_ids: Vec<i32> = activity_refs
            .iter()
            .map(|r| r.title_group_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Step 3: Title group data
        let title_groups = sqlx::query_as!(
            TitleGroupHierarchyLite,
            r#"
            SELECT DISTINCT ON (title_group_id)
                title_group_id AS "id!",
                title_group_name AS "name!",
                title_group_covers AS "covers!",
                title_group_category AS "category!: _",
                title_group_content_type AS "content_type!: _",
                title_group_tag_names AS "tags!",
                title_group_original_release_date AS "original_release_date",
                title_group_original_release_date_only_year_known AS "original_release_date_only_year_known!",
                title_group_platform AS "platform!: _",
                '[]'::jsonb AS "edition_groups!: _",
                '[]'::jsonb AS "affiliated_artists!: _",
                CASE
                    WHEN title_group_series_id IS NOT NULL THEN jsonb_build_object('id', title_group_series_id, 'name', title_group_series_name)
                    ELSE NULL
                END AS "series: _"
            FROM title_group_hierarchy_lite
            WHERE title_group_id = ANY($1)
            "#,
            &title_group_ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| Error::ErrorSearchingForTorrents(error.to_string()))?;

        let mut title_group_map: HashMap<i32, TitleGroupHierarchyLite> = HashMap::new();
        for tg in title_groups {
            title_group_map.insert(tg.id, tg);
        }

        // Step 4: Edition groups
        let edition_groups = sqlx::query_as!(
            EditionGroupHierarchyLite,
            r#"
            SELECT
                id,
                title_group_id,
                name,
                release_date,
                release_date_only_year_known,
                distributor,
                covers,
                source AS "source: _",
                additional_information AS "additional_information: _",
                '[]'::jsonb AS "torrents!: _"
            FROM edition_groups
            WHERE id = ANY($1)
            "#,
            &edition_group_ids
        )
        .fetch_all(&self.pool)
        .await?;

        let mut edition_group_map: HashMap<i32, EditionGroupHierarchyLite> = HashMap::new();
        for eg in edition_groups {
            edition_group_map.insert(eg.id, eg);
        }

        // Step 5: Torrents
        let torrents = sqlx::query_as!(
            TorrentHierarchyLite,
            r#"
            SELECT
                tar.id AS "id!",
                tar.upload_factor AS "upload_factor!",
                tar.download_factor AS "download_factor!",
                tar.seeders AS "seeders!",
                tar.leechers AS "leechers!",
                tar.times_completed AS "times_completed!",
                tar.grabbed AS "grabbed!",
                tar.edition_group_id AS "edition_group_id!",
                tar.created_at AS "created_at!: _",
                CASE
                    WHEN tar.uploaded_as_anonymous AND tar.created_by_id != $2 THEN
                        NULL
                    ELSE
                        ROW(u.id, u.username, u.warned, u.banned)
                END AS "created_by: UserLite",
                tar.uploaded_as_anonymous AS "uploaded_as_anonymous!",
                tar.release_name,
                tar.release_group,
                tar.trumpable,
                tar.staff_checked AS "staff_checked!",
                COALESCE(tar.languages, '{}') AS "languages!: _",
                tar.container AS "container!",
                tar.size AS "size!",
                tar.duration,
                tar.audio_codec AS "audio_codec: _",
                tar.audio_bitrate,
                tar.audio_bitrate_sampling AS "audio_bitrate_sampling: _",
                tar.audio_channels AS "audio_channels: _",
                tar.video_codec AS "video_codec: _",
                tar.features AS "features!: _",
                COALESCE(tar.subtitle_languages, '{}') AS "subtitle_languages!: _",
                tar.video_resolution AS "video_resolution: _",
                tar.video_resolution_other_x,
                tar.video_resolution_other_y,
                tar.extra_text,
                tar.reports AS "reports!: _",
                COALESCE(tar.extras, '{}') AS "extras!: _",
                CASE
                    WHEN EXISTS (
                        SELECT 1 FROM peers
                        WHERE torrent_id = tar.id
                        AND user_id = $2
                        AND active = true
                        AND seeder = true
                    ) THEN 'seeding'
                    WHEN EXISTS (
                        SELECT 1 FROM peers
                        WHERE torrent_id = tar.id
                        AND user_id = $2
                        AND active = true
                        AND seeder = false
                    ) THEN 'leeching'
                    WHEN EXISTS (
                        SELECT 1 FROM torrent_activities
                        WHERE torrent_id = tar.id
                        AND user_id = $2
                        AND completed_at IS NOT NULL
                    ) THEN 'snatched'
                    WHEN EXISTS (
                        SELECT 1 FROM torrent_activities
                        WHERE torrent_id = tar.id
                        AND user_id = $2
                        AND grabbed_at IS NOT NULL
                    ) AND NOT EXISTS (
                        SELECT 1 FROM peers
                        WHERE torrent_id = tar.id
                        AND user_id = $2
                        AND active = true
                    ) THEN 'grabbed'
                    ELSE NULL
                END AS "peer_status: _",
                tar.bonus_points_snatch_cost AS "bonus_points_snatch_cost!",
                tar.upload_method AS "upload_method!"
            FROM torrents_and_reports tar
            JOIN users u ON tar.created_by_id = u.id
            WHERE tar.id = ANY($1)
            ORDER BY tar.size DESC
            "#,
            &torrent_ids,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut torrent_map: HashMap<i32, TorrentHierarchyLite> = HashMap::new();
        for t in torrents {
            torrent_map.insert(t.id, t);
        }

        // Step 6: Torrent activities
        let activities_query = format!(
            r#"
            SELECT ta.id, ta.torrent_id, ta.user_id, ta.grabbed_at, ta.completed_at,
                   ta.first_seen_seeding_at, ta.last_seen_seeding_at, ta.total_seed_time,
                   ta.bonus_points, ta.uploaded, ta.real_uploaded, ta.downloaded, ta.real_downloaded,
                   EXISTS (
                       SELECT 1 FROM peers p
                       WHERE p.torrent_id = ta.torrent_id
                         AND p.user_id = ta.user_id
                         AND p.seeder = true
                         AND p.active = true
                   ) AS seeder,
                   COALESCE(
                       (SELECT ROUND({formula})::bigint * $3
                        FROM torrents t
                        INNER JOIN peers p ON p.torrent_id = t.id AND p.user_id = ta.user_id
                        WHERE t.id = ta.torrent_id AND t.staff_checked AND p.seeder = true AND p.active = true
                        LIMIT 1),
                       0
                   ) AS bonus_points_per_day
            FROM torrent_activities ta
            WHERE ta.user_id = $1 AND ta.torrent_id = ANY($2)
            "#,
            formula = formula_sql
        );
        let activities: Vec<TorrentActivity> =
            sqlx::query_as(sqlx::AssertSqlSafe(activities_query))
                .bind(user_id)
                .bind(&torrent_ids)
                .bind(ticks_per_day)
                .fetch_all(&self.pool)
                .await?;

        let mut activity_map: HashMap<i32, TorrentActivity> = HashMap::new();
        for activity in activities {
            activity_map.insert(activity.torrent_id, activity);
        }

        // Step 7: Affiliated artists
        let grouped_artists = self
            .find_affiliated_artists_lite_by_title_group_ids(&title_group_ids)
            .await?;

        // Step 8: Assembly
        let mut results = Vec::new();
        for activity_ref in &activity_refs {
            let Some(title_group) = title_group_map.get(&activity_ref.title_group_id) else {
                continue;
            };
            let Some(edition_group) = edition_group_map.get(&activity_ref.edition_group_id) else {
                continue;
            };
            let Some(torrent) = torrent_map.remove(&activity_ref.torrent_id) else {
                continue;
            };
            let Some(activity) = activity_map.remove(&activity_ref.torrent_id) else {
                continue;
            };

            let mut edition_group = edition_group.clone();
            edition_group.torrents = Json(vec![torrent]);

            let mut title_group = title_group.clone();
            title_group.edition_groups = Json(vec![edition_group]);
            title_group.affiliated_artists = Json(
                grouped_artists
                    .get(&title_group.id)
                    .cloned()
                    .unwrap_or_default(),
            );

            results.push(TorrentActivityAndTitleGroup {
                title_group,
                torrent_activity: activity,
            });
        }

        Ok(PaginatedResults {
            results,
            page: query.page,
            page_size: query.page_size,
            total_items: total_count.unwrap_or(0),
        })
    }

    pub async fn get_torrent_activities_overview(
        &self,
        user_id: i32,
        formula_sql: &str,
    ) -> Result<i64> {
        let query = format!(
            r#"
            SELECT COALESCE(SUM(ROUND({formula}))::bigint, 0)
            FROM torrent_activities ta
            INNER JOIN torrents t ON ta.torrent_id = t.id
            INNER JOIN peers p ON p.torrent_id = t.id AND p.user_id = ta.user_id
            WHERE t.staff_checked AND p.seeder = true AND p.active = true AND ta.user_id = $1
            "#,
            formula = formula_sql
        );

        let (bonus_per_tick,): (i64,) = sqlx::query_as(sqlx::AssertSqlSafe(query))
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(bonus_per_tick)
    }

    pub async fn get_torrent_title_group_id(&self, torrent_id: i32) -> Result<i32> {
        let title_group_id = sqlx::query_scalar!(
            r#"
            SELECT eg.title_group_id
            FROM torrents t
            JOIN edition_groups eg ON t.edition_group_id = eg.id
            WHERE t.id = $1 AND t.deleted_at IS NULL
            "#,
            torrent_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::TorrentNotFound)?;

        Ok(title_group_id)
    }

    /// Display name of the title group a torrent belongs to, prefixed with its series if any.
    pub async fn get_torrent_title_group_name(&self, torrent_id: i32) -> Result<String> {
        let title_group = sqlx::query!(
            r#"
            SELECT tg.name, s.name AS "series_name?"
            FROM torrents t
            JOIN edition_groups eg ON t.edition_group_id = eg.id
            JOIN title_groups tg ON tg.id = eg.title_group_id
            LEFT JOIN series s ON s.id = tg.series_id
            WHERE t.id = $1 AND t.deleted_at IS NULL
            "#,
            torrent_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::TorrentNotFound)?;

        Ok(format_title_group_name(
            title_group.series_name.as_deref(),
            &title_group.name,
        ))
    }
}
