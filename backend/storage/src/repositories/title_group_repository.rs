use crate::{
    connection_pool::ConnectionPool,
    models::{
        artist::{AffiliatedArtistHierarchy, Artist, ArtistRole},
        collage::{CollageCategory, CollageSearchResult},
        edition_group::{EditionGroup, EditionGroupHierarchy, Source},
        entity::{AffiliatedEntityHierarchy, Entity, EntityRole},
        series::SeriesLite,
        title_group::{
            ContentType, EditedTitleGroup, MasterGroupEntry, Platform, PublicRating,
            SimilarTitleGroupLite, SimilarTitleGroupsLink, TitleGroup, TitleGroupAndAssociatedData,
            TitleGroupCategory, UserCreatedTitleGroup,
        },
        title_group_comment::TitleGroupCommentHierarchy,
        title_group_tag::UserCreatedTitleGroupTag,
        torrent::{
            AudioBitrateSampling, AudioChannels, AudioCodec, Extras, Features, Language,
            PeerStatus, TorrentHierarchy, VideoCodec, VideoResolution,
        },
        torrent_report::TorrentReport,
        torrent_request::{TorrentRequest, TorrentRequestBounty, TorrentRequestHierarchyLite},
        user::{UserLite, UserLiteAvatar},
    },
};
use arcadia_common::error::{Error, Result};
use chrono::Local;
use serde_json::{json, Value};
use sqlx::types::Json;
use std::collections::HashMap;

impl ConnectionPool {
    pub async fn create_title_group(
        &self,
        title_group_form: &UserCreatedTitleGroup,
        public_ratings: &Vec<PublicRating>,
        user_id: i32,
    ) -> Result<TitleGroup> {
        let created_title_group_id: i32 = sqlx::query_scalar!(
            r#"
            INSERT INTO title_groups (
                master_group_id,
                name,
                name_aliases,
                created_by_id,
                description,
                original_language,
                country_from,
                covers,
                external_links,
                trailers,
                category,
                content_type,
                original_release_date,
                original_release_date_only_year_known,
                tagline,
                platform,
                screenshots,
                public_ratings
            )
            VALUES (
                $1, $2, $3, $4, $5, $6::language_enum,
                $7, $8, $9, $10, $11::title_group_category_enum,
                $12::content_type_enum, $13, $14, $15, $16, $17, $18
            )
            RETURNING id
            "#,
            title_group_form.master_group_id,
            &title_group_form.name,
            &title_group_form.name_aliases,
            user_id,
            &title_group_form.description,
            title_group_form.original_language.clone() as Option<Language>,
            title_group_form.country_from,
            &title_group_form.covers,
            &title_group_form.external_links,
            &title_group_form.trailers,
            title_group_form.category.clone() as Option<TitleGroupCategory>,
            title_group_form.content_type.clone() as ContentType,
            title_group_form.original_release_date,
            title_group_form.original_release_date_only_year_known,
            title_group_form.tagline,
            title_group_form.platform.clone() as Option<Platform>,
            &title_group_form.screenshots,
            json!(public_ratings)
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateTitleGroup)?;

        // Increment user's title_groups counter
        sqlx::query!(
            r#"
            UPDATE users
            SET title_groups = title_groups + 1
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        // ensure tags exist
        let mut tag_ids = Vec::new();
        for tag_name in title_group_form.tags.iter() {
            let tag = Self::create_title_group_tag(
                self,
                &UserCreatedTitleGroupTag {
                    name: tag_name.clone(),
                },
                user_id,
            )
            .await?;
            tag_ids.push(tag.id);
        }

        // apply tags to title group
        for tag_id in tag_ids {
            Self::apply_tag_to_title_group(self, created_title_group_id, tag_id, user_id).await?;
        }

        let created_title_group = Self::find_title_group(self, created_title_group_id).await?;

        Ok(created_title_group)
    }

    pub async fn find_title_group_hierarchy(
        &self,
        title_group_id: i32,
        user_id: i32,
    ) -> Result<TitleGroupAndAssociatedData> {
        let title_group = self.find_title_group(title_group_id).await?;

        let (
            edition_group_rows,
            torrent_rows,
            report_rows,
            affiliated_artist_rows,
            affiliated_entity_rows,
            comment_rows,
            torrent_request_rows,
            series,
            subscriptions,
            master_group_entries,
            collage_rows,
            active_peer_rows,
            torrent_activity_rows,
        ) = tokio::try_join!(
            // Edition groups
            sqlx::query_as!(
                EditionGroup,
                r#"
                SELECT
                    id, title_group_id, name, release_date, release_date_only_year_known,
                    created_at, updated_at, created_by_id, description, distributor,
                    covers, external_links, source AS "source: Source",
                    additional_information
                FROM edition_groups
                WHERE title_group_id = $1
                ORDER BY release_date
                "#,
                title_group_id
            )
            .fetch_all(&self.pool),
            // Torrents with user info
            sqlx::query!(
                r#"
                SELECT
                    t.id, t.upload_factor, t.download_factor, t.seeders, t.leechers,
                    t.times_completed, t.grabbed, t.edition_group_id,
                    t.created_at, t.updated_at, t.created_by_id,
                    t.extras AS "extras: Vec<Extras>",
                    t.release_name, t.release_group, t.description,
                    t.file_amount_per_type, t.uploaded_as_anonymous, t.file_list,
                    t.mediainfo, t.trumpable, t.staff_checked,
                    t.languages AS "languages: Vec<Language>",
                    t.container, t.size, t.duration,
                    t.audio_codec AS "audio_codec: AudioCodec",
                    t.audio_bitrate,
                    t.audio_bitrate_sampling AS "audio_bitrate_sampling: AudioBitrateSampling",
                    t.audio_channels AS "audio_channels: AudioChannels",
                    t.video_codec AS "video_codec: VideoCodec",
                    t.features AS "features: Vec<Features>",
                    t.subtitle_languages AS "subtitle_languages: Vec<Language>",
                    t.video_resolution AS "video_resolution: VideoResolution",
                    t.video_resolution_other_x, t.video_resolution_other_y,
                    t.extra_text, t.bonus_points_snatch_cost, t.upload_method,
                    CASE WHEN t.seeders = 0
                         THEN (SELECT MAX(ta.last_seen_seeding_at)
                               FROM torrent_activities ta WHERE ta.torrent_id = t.id)
                    END AS "last_seeded_at?",
                    CASE WHEN t.seeders = 0
                         THEN (SELECT MAX(nrr.created_at)
                               FROM notifications_reseed_requests nrr
                               WHERE nrr.torrent_id = t.id AND nrr.requested_by_id = $2)
                    END AS "reseed_request_sent_at?",
                    u.id AS "user_id?", u.username AS "user_username?",
                    u.warned AS "user_warned?", u.banned AS "user_banned?"
                FROM torrents t
                LEFT JOIN users u ON u.id = t.created_by_id
                WHERE t.edition_group_id IN (SELECT id FROM edition_groups WHERE title_group_id = $1)
                  AND t.deleted_at IS NULL
                ORDER BY t.size DESC
                "#,
                title_group_id,
                user_id
            )
            .fetch_all(&self.pool),
            // Torrent reports
            sqlx::query_as!(
                TorrentReport,
                r#"
                SELECT tr.id, tr.reported_at, tr.reported_torrent_id, tr.description,
                    json_build_object('id', u.id, 'username', u.username, 'warned', u.warned, 'banned', u.banned) AS "reported_by!: Json<UserLite>"
                FROM torrent_reports tr
                JOIN users u ON u.id = tr.reported_by_id
                WHERE tr.reported_torrent_id IN (
                    SELECT t.id FROM torrents t
                    JOIN edition_groups eg ON eg.id = t.edition_group_id
                    WHERE eg.title_group_id = $1 AND t.deleted_at IS NULL
                )
                "#,
                title_group_id
            )
            .fetch_all(&self.pool),
            // Affiliated artists with artist data
            sqlx::query!(
                r#"
                SELECT
                    aa.id, aa.title_group_id, aa.artist_id,
                    aa.roles AS "roles: Vec<ArtistRole>",
                    aa.nickname, aa.created_at, aa.created_by_id,
                    a.id AS a_id, a.name AS a_name, a.aliases AS a_aliases,
                    a.created_at AS a_created_at,
                    a.created_by_id AS a_created_by_id, a.description AS a_description,
                    a.pictures AS a_pictures, a.external_links AS a_external_links,
                    a.title_groups_amount AS a_title_groups_amount,
                    a.edition_groups_amount AS a_edition_groups_amount,
                    a.torrents_amount AS a_torrents_amount,
                    a.seeders_amount AS a_seeders_amount,
                    a.leechers_amount AS a_leechers_amount,
                    a.snatches_amount AS a_snatches_amount,
                    a.total_size AS a_total_size
                FROM affiliated_artists aa
                JOIN artists a ON a.id = aa.artist_id
                WHERE aa.title_group_id = $1
                "#,
                title_group_id
            )
            .fetch_all(&self.pool),
            // Affiliated entities with entity data
            sqlx::query!(
                r#"
                SELECT
                    ae.id, ae.title_group_id, ae.entity_id, ae.created_by_id, ae.created_at,
                    ae.roles AS "roles: Vec<EntityRole>",
                    e.id AS e_id, e.name AS e_name, e.created_at AS e_created_at,
                    e.created_by_id AS e_created_by_id, e.description AS e_description,
                    e.pictures AS e_pictures
                FROM affiliated_entities ae
                JOIN entities e ON e.id = ae.entity_id
                WHERE ae.title_group_id = $1
                "#,
                title_group_id
            )
            .fetch_all(&self.pool),
            // Comments with user data
            sqlx::query!(
                r#"
                SELECT
                    c.id, c.content, c.created_at, c.updated_at, c.created_by_id,
                    c.title_group_id, c.locked, c.refers_to_torrent_id, c.answers_to_comment_id,
                    u.id AS "u_id!", u.username AS "u_username!", u.class_name AS "u_class_name!",
                    u.banned AS "u_banned!", u.avatar AS "u_avatar?", u.warned AS "u_warned!",
                    u.custom_title AS "u_custom_title?"
                FROM title_group_comments c
                JOIN users u ON u.id = c.created_by_id
                WHERE c.title_group_id = $1
                ORDER BY c.created_at
                "#,
                title_group_id
            )
            .fetch_all(&self.pool),
            // Torrent requests with bounty and user data
            sqlx::query!(
                r#"
                SELECT
                    tr.id, tr.title_group_id, tr.created_at, tr.updated_at, tr.created_by_id,
                    tr.filled_by_user_id, tr.filled_by_torrent_id, tr.filled_at,
                    tr.edition_name,
                    tr.source AS "source: Vec<Source>",
                    tr.release_group, tr.description,
                    tr.languages AS "languages: Vec<Language>",
                    tr.container,
                    tr.audio_codec AS "audio_codec: Vec<AudioCodec>",
                    tr.audio_channels AS "audio_channels: Vec<AudioChannels>",
                    tr.audio_bitrate_sampling AS "audio_bitrate_sampling: Vec<AudioBitrateSampling>",
                    tr.video_codec AS "video_codec: Vec<VideoCodec>",
                    tr.features AS "features: Vec<Features>",
                    tr.subtitle_languages AS "subtitle_languages: Vec<Language>",
                    tr.video_resolution AS "video_resolution: Vec<VideoResolution>",
                    tr.video_resolution_other_x, tr.video_resolution_other_y,
                    u.id AS "u_id!", u.username AS "u_username!",
                    u.warned AS "u_warned!", u.banned AS "u_banned!",
                    fu.id AS "fu_id?", fu.username AS "fu_username?",
                    fu.warned AS "fu_warned?", fu.banned AS "fu_banned?",
                    COALESCE(SUM(trv.bounty_upload), 0)::BIGINT AS "total_upload_bounty!: i64",
                    COALESCE(SUM(trv.bounty_bonus_points), 0)::BIGINT AS "total_bonus_bounty!: i64",
                    COUNT(DISTINCT trv.created_by_id) AS "user_votes_amount!"
                FROM torrent_requests tr
                LEFT JOIN torrent_request_votes trv ON tr.id = trv.torrent_request_id
                JOIN users u ON u.id = tr.created_by_id
                LEFT JOIN users fu ON fu.id = tr.filled_by_user_id
                WHERE tr.title_group_id = $1
                GROUP BY tr.id, u.id, u.username, u.warned, u.banned,
                         fu.id, fu.username, fu.warned, fu.banned
                ORDER BY tr.id
                "#,
                title_group_id
            )
            .fetch_all(&self.pool),
            // Series
            async {
                match title_group.series_id {
                    Some(series_id) => sqlx::query_as!(
                        SeriesLite,
                        r#"SELECT id, name FROM series WHERE id = $1"#,
                        series_id
                    )
                    .fetch_optional(&self.pool)
                    .await,
                    None => Ok(None),
                }
            },
            // Subscriptions
            sqlx::query!(
                r#"
                SELECT
                    EXISTS(
                        SELECT 1 FROM subscriptions_title_group_torrents
                        WHERE title_group_id = $1 AND user_id = $2
                    ) AS "is_subscribed_to_torrents!",
                    EXISTS(
                        SELECT 1 FROM subscriptions_title_group_comments
                        WHERE title_group_id = $1 AND user_id = $2
                    ) AS "is_subscribed_to_comments!",
                    (
                        SELECT id FROM title_group_bookmarks
                        WHERE title_group_id = $1 AND user_id = $2
                        LIMIT 1
                    ) AS bookmark_id
                "#,
                title_group_id,
                user_id
            )
            .fetch_one(&self.pool),
            // Same master group entries
            async {
                match title_group.master_group_id {
                    Some(master_group_id) => sqlx::query_as!(
                        MasterGroupEntry,
                        r#"
                        SELECT
                            id, name,
                            content_type AS "content_type: ContentType",
                            platform AS "platform: Platform"
                        FROM title_groups
                        WHERE master_group_id = $1 AND id != $2
                        "#,
                        master_group_id,
                        title_group_id
                    )
                    .fetch_all(&self.pool)
                    .await,
                    None => Ok(vec![]),
                }
            },
            // Collages
            sqlx::query!(
                r#"
                SELECT
                    c.id, c.created_at, c.created_by_id,
                    u.id AS "u_id!", u.username AS "u_username!",
                    u.warned AS "u_warned!", u.banned AS "u_banned!",
                    c.name, c.cover, c.description, c.tags,
                    c.category AS "category: CollageCategory",
                    COUNT(ce.id)::BIGINT AS "entries_amount!",
                    MAX(ce.created_at) AS last_entry_at
                FROM collage c
                JOIN collage_entry ce ON ce.collage_id = c.id
                JOIN users u ON u.id = c.created_by_id
                WHERE c.id IN (
                    SELECT collage_id FROM collage_entry WHERE title_group_id = $1
                )
                GROUP BY c.id, u.id, u.username, u.warned, u.banned
                ORDER BY c.created_at
                "#,
                title_group_id
            )
            .fetch_all(&self.pool),
            // Active peers for the current user on this title group's torrents
            sqlx::query!(
                r#"
                SELECT p.torrent_id, p.seeder
                FROM peers p
                WHERE p.user_id = $2 AND p.active = true
                AND p.torrent_id IN (
                    SELECT t.id FROM torrents t
                    JOIN edition_groups eg ON eg.id = t.edition_group_id
                    WHERE eg.title_group_id = $1 AND t.deleted_at IS NULL
                )
                "#,
                title_group_id,
                user_id
            )
            .fetch_all(&self.pool),
            // Torrent activities for the current user on this title group's torrents
            sqlx::query!(
                r#"
                SELECT
                    ta.torrent_id,
                    ta.completed_at IS NOT NULL AS "completed!",
                    ta.grabbed_at IS NOT NULL AS "grabbed!"
                FROM torrent_activities ta
                WHERE ta.user_id = $2
                AND ta.torrent_id IN (
                    SELECT t.id FROM torrents t
                    JOIN edition_groups eg ON eg.id = t.edition_group_id
                    WHERE eg.title_group_id = $1 AND t.deleted_at IS NULL
                )
                "#,
                title_group_id,
                user_id
            )
            .fetch_all(&self.pool),
        )?;

        // Group reports by torrent_id
        let mut reports_by_torrent: HashMap<i32, Vec<TorrentReport>> =
            HashMap::with_capacity(report_rows.len());
        for report in report_rows {
            reports_by_torrent
                .entry(report.reported_torrent_id)
                .or_default()
                .push(report);
        }

        // Build peer status lookup from batch queries
        let mut active_peers_by_torrent: HashMap<i32, bool> =
            HashMap::with_capacity(active_peer_rows.len());
        for row in &active_peer_rows {
            let entry = active_peers_by_torrent.entry(row.torrent_id);
            // Prefer seeder=true if multiple peer entries exist
            entry
                .and_modify(|seeder| {
                    if row.seeder {
                        *seeder = true;
                    }
                })
                .or_insert(row.seeder);
        }

        let mut activities_by_torrent: HashMap<i32, (bool, bool)> =
            HashMap::with_capacity(torrent_activity_rows.len());
        for row in torrent_activity_rows {
            activities_by_torrent.insert(row.torrent_id, (row.completed, row.grabbed));
        }

        // Build TorrentHierarchy rows grouped by edition_group_id
        let mut torrents_by_edition_group: HashMap<i32, Vec<TorrentHierarchy>> =
            HashMap::with_capacity(torrent_rows.len());
        for row in torrent_rows {
            let is_anonymous = row.uploaded_as_anonymous;
            let is_own_upload = row.created_by_id == user_id;

            let (created_by_id, created_by) = if is_anonymous && !is_own_upload {
                (None, None)
            } else {
                (
                    Some(row.created_by_id),
                    row.user_id.map(|id| UserLite {
                        id,
                        username: row.user_username.unwrap_or_default(),
                        warned: row.user_warned.unwrap_or_default(),
                        banned: row.user_banned.unwrap_or_default(),
                    }),
                )
            };

            let torrent = TorrentHierarchy {
                id: row.id,
                upload_factor: row.upload_factor,
                download_factor: row.download_factor,
                seeders: row.seeders,
                leechers: row.leechers,
                last_seeded_at: row.last_seeded_at.map(|d| d.with_timezone(&Local)),
                reseed_request_sent_at: row.reseed_request_sent_at.map(|d| d.with_timezone(&Local)),
                times_completed: row.times_completed,
                grabbed: row.grabbed,
                edition_group_id: row.edition_group_id,
                created_at: row.created_at.with_timezone(&Local),
                updated_at: row.updated_at.with_timezone(&Local),
                created_by_id,
                created_by,
                extras: row.extras.unwrap_or_default(),
                release_name: Some(row.release_name),
                release_group: row.release_group,
                description: row.description,
                file_amount_per_type: row.file_amount_per_type.into(),
                uploaded_as_anonymous: row.uploaded_as_anonymous,
                file_list: row.file_list.into(),
                mediainfo: row.mediainfo,
                trumpable: row.trumpable,
                staff_checked: row.staff_checked,
                languages: row.languages,
                container: row.container,
                size: row.size,
                duration: row.duration,
                audio_codec: row.audio_codec,
                audio_bitrate: row.audio_bitrate,
                audio_bitrate_sampling: row.audio_bitrate_sampling,
                audio_channels: row.audio_channels,
                video_codec: row.video_codec,
                features: row.features,
                subtitle_languages: row.subtitle_languages,
                video_resolution: row.video_resolution,
                video_resolution_other_x: row.video_resolution_other_x,
                video_resolution_other_y: row.video_resolution_other_y,
                extra_text: row.extra_text,
                reports: reports_by_torrent.remove(&row.id).unwrap_or_default(),
                peer_status: if let Some(&is_seeder) = active_peers_by_torrent.get(&row.id) {
                    if is_seeder {
                        Some(PeerStatus::Seeding)
                    } else {
                        Some(PeerStatus::Leeching)
                    }
                } else if let Some(&(completed, grabbed)) = activities_by_torrent.get(&row.id) {
                    if completed {
                        Some(PeerStatus::Snatched)
                    } else if grabbed {
                        Some(PeerStatus::Grabbed)
                    } else {
                        None
                    }
                } else {
                    None
                },
                bonus_points_snatch_cost: row.bonus_points_snatch_cost,
                upload_method: row.upload_method,
            };

            torrents_by_edition_group
                .entry(row.edition_group_id)
                .or_default()
                .push(torrent);
        }

        // Build edition group hierarchy
        let edition_groups: Vec<EditionGroupHierarchy> = edition_group_rows
            .into_iter()
            .map(|eg| EditionGroupHierarchy {
                id: eg.id,
                title_group_id: eg.title_group_id,
                name: eg.name,
                release_date: eg.release_date,
                release_date_only_year_known: eg.release_date_only_year_known,
                created_at: eg.created_at,
                updated_at: eg.updated_at,
                created_by_id: eg.created_by_id,
                description: eg.description,
                distributor: eg.distributor,
                covers: eg.covers,
                external_links: eg.external_links,
                source: eg.source,
                additional_information: eg.additional_information.map(Into::into),
                torrents: torrents_by_edition_group.remove(&eg.id).unwrap_or_default(),
            })
            .collect();

        // Build affiliated artists
        let affiliated_artists: Vec<AffiliatedArtistHierarchy> = affiliated_artist_rows
            .into_iter()
            .map(|row| AffiliatedArtistHierarchy {
                id: row.id,
                title_group_id: row.title_group_id,
                artist_id: row.artist_id,
                roles: row.roles,
                nickname: row.nickname,
                created_at: row.created_at,
                created_by_id: row.created_by_id,
                artist: Artist {
                    id: row.a_id,
                    name: row.a_name,
                    aliases: row.a_aliases,
                    created_at: row.a_created_at,
                    created_by_id: row.a_created_by_id,
                    description: row.a_description,
                    pictures: row.a_pictures,
                    external_links: row.a_external_links,
                    title_groups_amount: row.a_title_groups_amount,
                    edition_groups_amount: row.a_edition_groups_amount,
                    torrents_amount: row.a_torrents_amount,
                    seeders_amount: row.a_seeders_amount,
                    leechers_amount: row.a_leechers_amount,
                    snatches_amount: row.a_snatches_amount,
                    total_size: row.a_total_size,
                },
            })
            .collect();

        // Build affiliated entities
        let affiliated_entities: Vec<AffiliatedEntityHierarchy> = affiliated_entity_rows
            .into_iter()
            .map(|row| AffiliatedEntityHierarchy {
                id: row.id,
                title_group_id: row.title_group_id,
                entity_id: row.entity_id,
                created_by_id: row.created_by_id,
                created_at: row.created_at.with_timezone(&Local),
                roles: row.roles,
                entity: Entity {
                    id: row.e_id,
                    name: row.e_name,
                    created_at: row.e_created_at.with_timezone(&Local),
                    created_by_id: row.e_created_by_id,
                    description: row.e_description,
                    pictures: row.e_pictures,
                },
            })
            .collect();

        // Build comments
        let title_group_comments: Vec<TitleGroupCommentHierarchy> = comment_rows
            .into_iter()
            .map(|row| TitleGroupCommentHierarchy {
                id: row.id,
                content: row.content,
                created_at: row.created_at.with_timezone(&Local),
                updated_at: row.updated_at.with_timezone(&Local),
                created_by_id: row.created_by_id,
                title_group_id: row.title_group_id,
                locked: row.locked,
                refers_to_torrent_id: row.refers_to_torrent_id,
                answers_to_comment_id: row.answers_to_comment_id,
                created_by: UserLiteAvatar {
                    id: row.u_id,
                    username: row.u_username,
                    class_name: row.u_class_name,
                    banned: row.u_banned,
                    avatar: row.u_avatar,
                    warned: row.u_warned,
                    custom_title: row.u_custom_title,
                },
            })
            .collect();

        // Build torrent requests
        let torrent_requests: Vec<TorrentRequestHierarchyLite> = torrent_request_rows
            .into_iter()
            .map(|row| TorrentRequestHierarchyLite {
                torrent_request: TorrentRequest {
                    id: row.id,
                    title_group_id: row.title_group_id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    created_by_id: row.created_by_id,
                    filled_by_user_id: row.filled_by_user_id,
                    filled_by_torrent_id: row.filled_by_torrent_id,
                    filled_at: row.filled_at,
                    edition_name: row.edition_name,
                    source: row.source,
                    release_group: row.release_group,
                    description: row.description,
                    languages: row.languages,
                    container: row.container,
                    audio_codec: row.audio_codec,
                    audio_channels: row.audio_channels,
                    audio_bitrate_sampling: row.audio_bitrate_sampling,
                    video_codec: row.video_codec,
                    features: row.features,
                    subtitle_languages: row.subtitle_languages,
                    video_resolution: row.video_resolution,
                    video_resolution_other_x: row.video_resolution_other_x,
                    video_resolution_other_y: row.video_resolution_other_y,
                },
                created_by: UserLite {
                    id: row.u_id,
                    username: row.u_username,
                    warned: row.u_warned,
                    banned: row.u_banned,
                },
                filled_by: row.fu_id.map(|id| UserLite {
                    id,
                    username: row.fu_username.unwrap_or_default(),
                    warned: row.fu_warned.unwrap_or_default(),
                    banned: row.fu_banned.unwrap_or_default(),
                }),
                user_votes_amount: row.user_votes_amount as i32,
                bounty: TorrentRequestBounty {
                    upload: row.total_upload_bounty,
                    bonus_points: row.total_bonus_bounty,
                },
            })
            .collect();

        // Build collages
        let collages: Vec<CollageSearchResult> = collage_rows
            .into_iter()
            .map(|row| CollageSearchResult {
                id: row.id,
                created_at: row.created_at.with_timezone(&Local),
                created_by_id: row.created_by_id,
                created_by: UserLite {
                    id: row.u_id,
                    username: row.u_username,
                    warned: row.u_warned,
                    banned: row.u_banned,
                },
                name: row.name,
                cover: row.cover,
                description: row.description,
                tags: row.tags,
                category: row.category,
                entries_amount: row.entries_amount,
                last_entry_at: row.last_entry_at.map(|dt| dt.with_timezone(&Local)),
            })
            .collect();

        let related_threads = self
            .find_related_forum_threads_for_title_group(title_group_id)
            .await?;

        let similar_title_groups = self.find_similar_title_groups(title_group_id).await?;

        Ok(TitleGroupAndAssociatedData {
            title_group,
            edition_groups,
            series,
            affiliated_artists,
            affiliated_entities,
            title_group_comments,
            torrent_requests,
            is_subscribed_to_torrents: subscriptions.is_subscribed_to_torrents,
            is_subscribed_to_comments: subscriptions.is_subscribed_to_comments,
            bookmark_id: subscriptions.bookmark_id,
            in_same_master_group: master_group_entries,
            collages,
            related_threads,
            similar_title_groups,
        })
    }

    pub async fn find_similar_title_groups(
        &self,
        title_group_id: i32,
    ) -> Result<Vec<SimilarTitleGroupLite>> {
        let similar_title_groups = sqlx::query_as!(
            SimilarTitleGroupLite,
            r#"
            SELECT
                other.id,
                other.name,
                other.original_release_date,
                other.covers[1] AS cover,
                stg.note
            FROM similar_title_groups stg
            JOIN title_groups other ON other.id = CASE
                WHEN stg.group_1_id = $1 THEN stg.group_2_id
                ELSE stg.group_1_id
            END
            WHERE stg.group_1_id = $1 OR stg.group_2_id = $1
            ORDER BY other.name
            "#,
            title_group_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(similar_title_groups)
    }

    pub async fn link_similar_title_groups(
        &self,
        link: &SimilarTitleGroupsLink,
        current_user_id: i32,
    ) -> Result<()> {
        if link.group_1 == link.group_2 {
            return Err(Error::TitleGroupCannotBeLinkedToItself);
        }

        let (group_1_id, group_2_id) = if link.group_1 < link.group_2 {
            (link.group_1, link.group_2)
        } else {
            (link.group_2, link.group_1)
        };

        sqlx::query!(
            r#"
            INSERT INTO similar_title_groups (group_1_id, group_2_id, note, created_by_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING
            "#,
            group_1_id,
            group_2_id,
            link.note,
            current_user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotLinkSimilarTitleGroups)?;

        Ok(())
    }

    pub async fn unlink_similar_title_groups(&self, link: &SimilarTitleGroupsLink) -> Result<()> {
        let (group_1_id, group_2_id) = if link.group_1 < link.group_2 {
            (link.group_1, link.group_2)
        } else {
            (link.group_2, link.group_1)
        };

        sqlx::query!(
            r#"
            DELETE FROM similar_title_groups
            WHERE group_1_id = $1 AND group_2_id = $2
            "#,
            group_1_id,
            group_2_id,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotUnlinkSimilarTitleGroups)?;

        Ok(())
    }
    pub async fn find_title_group_info_lite(
        &self,
        title_group_id: Option<i32>,
        title_group_name: Option<&str>,
        title_group_content_type: &Option<ContentType>,
        limit: u32,
    ) -> Result<Value> {
        let title_groups = sqlx::query!(
            r#"
            WITH matching_series AS (
                -- Find series that match the search query
                SELECT id
                FROM series
                WHERE $2::TEXT IS NOT NULL AND name ILIKE '%' || $2 || '%'
            ),
            dynamic_limit AS (
                -- Use a higher limit (50) if any series match, otherwise use the provided limit
                SELECT CASE
                    WHEN EXISTS (SELECT 1 FROM matching_series) AND $2 != '' THEN 50
                    ELSE $4
                END AS result_limit
            ),
            latest_torrent_per_title_group AS (
                -- Find the latest torrent for each title group with uploader info
                SELECT DISTINCT ON (eg.title_group_id)
                    eg.title_group_id,
                    t.created_at AS torrent_created_at,
                    t.created_by_id,
                    t.uploaded_as_anonymous,
                    u.id AS user_id,
                    u.username,
                    u.warned,
                    u.banned
                FROM torrents t
                JOIN edition_groups eg ON eg.id = t.edition_group_id
                JOIN users u ON u.id = t.created_by_id
                WHERE t.deleted_at IS NULL
                ORDER BY eg.title_group_id, t.created_at DESC
            )
            SELECT jsonb_agg(data)
                FROM (
                    SELECT jsonb_build_object(
                        'id', tg.id, 'content_type', tg.content_type, 'name', tg.name, 'platform', tg.platform, 'covers', tg.covers,
                        'original_release_date', tg.original_release_date,
                        'original_release_date_only_year_known', tg.original_release_date_only_year_known,
                        'edition_groups', COALESCE(
                            jsonb_agg(
                                jsonb_build_object(
                                    'id', eg.id,
                                    'name', eg.name,
                                    'release_date', eg.release_date,
                                    'release_date_only_year_known', eg.release_date_only_year_known,
                                    'distributor', eg.distributor,
                                    'source', eg.source,
                                    'additional_information', eg.additional_information
                                )
                            ) FILTER (WHERE eg.id IS NOT NULL),
                            '[]'::jsonb
                        ),
                        'series', CASE WHEN s.id IS NOT NULL THEN
                            jsonb_build_object('id', s.id, 'name', s.name)
                            ELSE NULL
                        END,
                        'latest_torrent_uploaded_by', CASE
                            WHEN ltu.uploaded_as_anonymous THEN NULL
                            WHEN ltu.user_id IS NOT NULL THEN
                                jsonb_build_object('id', ltu.user_id, 'username', ltu.username, 'warned', ltu.warned, 'banned', ltu.banned)
                            ELSE NULL
                        END,
                        'latest_torrent_uploaded_at', ltu.torrent_created_at
                    ) as data
                    FROM title_groups tg
                    LEFT JOIN edition_groups eg ON eg.title_group_id = tg.id
                    LEFT JOIN series s ON s.id = tg.series_id
                    LEFT JOIN latest_torrent_per_title_group ltu ON ltu.title_group_id = tg.id
                    LEFT JOIN (
                        SELECT edition_group_id, MAX(created_at) as created_at
                        FROM torrents
                        GROUP BY edition_group_id
                    ) AS latest_torrent ON latest_torrent.edition_group_id = eg.id
                    WHERE ($1::INT IS NOT NULL AND tg.id = $1)
                        OR (
                            $2::TEXT IS NOT NULL
                            AND (
                                tg.name ILIKE '%' || $2 || '%'
                                OR EXISTS (SELECT 1 FROM unnest(tg.name_aliases) alias WHERE alias ILIKE '%' || $2 || '%')
                                OR (s.name IS NOT NULL AND s.name ILIKE '%' || $2 || '%')
                            )
                        )
                        AND ($3::content_type_enum IS NULL OR tg.content_type = $3::content_type_enum)
                    GROUP BY tg.id, s.id, s.name, ltu.uploaded_as_anonymous, ltu.user_id, ltu.username, ltu.warned, ltu.banned, ltu.torrent_created_at
                    ORDER BY MAX(latest_torrent.created_at) DESC NULLS LAST
                    LIMIT (SELECT result_limit FROM dynamic_limit)
                ) AS subquery;
            "#,
            title_group_id,
            title_group_name,
            title_group_content_type as &Option<ContentType>,
            limit as i32
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(title_groups
            .jsonb_agg
            .unwrap_or_else(|| serde_json::Value::Array(vec![])))
    }

    pub async fn find_title_group(&self, title_group_id: i32) -> Result<TitleGroup> {
        let title_group = sqlx::query_as!(
            TitleGroup,
            r#"
            SELECT
                id, master_group_id, name, name_aliases AS "name_aliases!: _",
                created_at, updated_at, created_by_id, description,
                platform AS "platform: _", original_language AS "original_language: _", original_release_date,
                original_release_date_only_year_known, tagline, country_from, covers AS "covers!: _",
                external_links AS "external_links!: _", trailers,
                category AS "category: _", content_type AS "content_type: _",
                public_ratings, screenshots AS "screenshots!: _", series_id,
                COALESCE(
                    ARRAY(
                        SELECT t.name
                        FROM title_group_applied_tags tat
                        JOIN title_group_tags t ON t.id = tat.tag_id
                        WHERE tat.title_group_id = title_groups.id
                    ),
                    ARRAY[]::text[]
                ) AS "tags!: _"
            FROM title_groups
            WHERE id = $1
            "#,
            title_group_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| Error::TitleGroupNotFound)?;

        Ok(title_group)
    }

    pub async fn update_title_group(
        &self,
        edited_title_group: &EditedTitleGroup,
        title_group_id: i32,
    ) -> Result<TitleGroup> {
        let updated_title_group = sqlx::query_as!(
            TitleGroup,
            r#"
            UPDATE title_groups
            SET
                master_group_id = $2,
                name = $3,
                name_aliases = $4,
                description = $5,
                platform = $6,
                original_language = $7,
                original_release_date = $8,
                original_release_date_only_year_known = $9,
                tagline = $10,
                country_from = $11,
                covers = $12,
                external_links = $13,
                trailers = $14,
                category = $15,
                content_type = $16,
                screenshots = $17,
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, master_group_id, name, name_aliases AS "name_aliases!: _",
                created_at, updated_at, created_by_id, description,
                platform AS "platform: _", original_language AS "original_language: _", original_release_date,
                original_release_date_only_year_known, tagline, country_from, covers AS "covers!: _",
                external_links AS "external_links!: _", trailers,
                category AS "category: _", content_type AS "content_type: _",
                public_ratings, screenshots AS "screenshots!: _", series_id,
                COALESCE(
                    ARRAY(
                        SELECT t.name
                        FROM title_group_applied_tags tat
                        JOIN title_group_tags t ON t.id = tat.tag_id
                        WHERE tat.title_group_id = title_groups.id
                    ),
                    ARRAY[]::text[]
                ) AS "tags!: _"
            "#,
            title_group_id,
            edited_title_group.master_group_id,
            edited_title_group.name,
            edited_title_group.name_aliases as _,
            edited_title_group.description,
            edited_title_group.platform as _,
            edited_title_group.original_language as _,
            edited_title_group.original_release_date,
            edited_title_group.original_release_date_only_year_known,
            edited_title_group.tagline,
            edited_title_group.country_from,
            edited_title_group.covers as _,
            edited_title_group.external_links as _,
            edited_title_group.trailers as _,
            edited_title_group.category as _,
            edited_title_group.content_type as _,
            edited_title_group.screenshots as _
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::ErrorWhileUpdatingTitleGroup(e.to_string()))?;

        Ok(updated_title_group)
    }

    pub async fn assign_title_group_to_series(
        &self,
        title_group_id: i32,
        series_id: i64,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
            UPDATE title_groups
            SET series_id = $2, updated_at = NOW()
            WHERE id = $1
            "#,
            title_group_id,
            series_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::ErrorWhileUpdatingTitleGroup(e.to_string()))?;

        Ok(())
    }

    pub async fn unassign_title_group_from_series(
        &self,
        title_group_id: i32,
        series_id: i64,
    ) -> Result<()> {
        let _ = sqlx::query!(
            r#"
            UPDATE title_groups
            SET series_id = NULL, updated_at = NOW()
            WHERE id = $1 AND series_id = $2
            "#,
            title_group_id,
            series_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::ErrorWhileUpdatingTitleGroup(e.to_string()))?;

        Ok(())
    }

    pub async fn does_title_group_with_link_exist(
        &self,
        external_link: &str,
    ) -> Result<Option<i32>> {
        let title_group_id: Option<i32> = sqlx::query_scalar!(
            r#"
            SELECT id
            FROM title_groups
            WHERE external_links @> ARRAY[$1::TEXT];
            "#,
            external_link
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::ErrorSearchingForTitleGroup(e.to_string()))?;

        Ok(title_group_id)
    }

    pub async fn merge_title_groups(
        &self,
        source_title_group_id: i32,
        target_title_group_id: i32,
    ) -> Result<()> {
        // Move edition_groups (no unique constraint on title_group_id)
        sqlx::query!(
            r#"
            UPDATE edition_groups
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move title_group_applied_tags (PK on title_group_id, tag_id)
        sqlx::query!(
            r#"
            DELETE FROM title_group_applied_tags
            WHERE title_group_id = $1
              AND tag_id IN (
                  SELECT tag_id FROM title_group_applied_tags WHERE title_group_id = $2
              )
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            UPDATE title_group_applied_tags
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move affiliated_artists (UNIQUE on title_group_id, artist_id)
        sqlx::query!(
            r#"
            DELETE FROM affiliated_artists
            WHERE title_group_id = $1
              AND artist_id IN (
                  SELECT artist_id FROM affiliated_artists WHERE title_group_id = $2
              )
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            UPDATE affiliated_artists
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move affiliated_entities (UNIQUE on title_group_id, entity_id)
        sqlx::query!(
            r#"
            DELETE FROM affiliated_entities
            WHERE title_group_id = $1
              AND entity_id IN (
                  SELECT entity_id FROM affiliated_entities WHERE title_group_id = $2
              )
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            UPDATE affiliated_entities
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move title_group_comments
        sqlx::query!(
            r#"
            UPDATE title_group_comments
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move torrent_requests
        sqlx::query!(
            r#"
            UPDATE torrent_requests
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move title_group_bookmarks
        sqlx::query!(
            r#"
            DELETE FROM title_group_bookmarks
            WHERE title_group_id = $1
              AND user_id IN (
                  SELECT user_id FROM title_group_bookmarks WHERE title_group_id = $2
              )
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            UPDATE title_group_bookmarks
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move subscriptions_title_group_torrents (UNIQUE on title_group_id, user_id)
        sqlx::query!(
            r#"
            DELETE FROM subscriptions_title_group_torrents
            WHERE title_group_id = $1
              AND user_id IN (
                  SELECT user_id FROM subscriptions_title_group_torrents WHERE title_group_id = $2
              )
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            UPDATE subscriptions_title_group_torrents
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move subscriptions_title_group_comments (UNIQUE on title_group_id, user_id)
        sqlx::query!(
            r#"
            DELETE FROM subscriptions_title_group_comments
            WHERE title_group_id = $1
              AND user_id IN (
                  SELECT user_id FROM subscriptions_title_group_comments WHERE title_group_id = $2
              )
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            UPDATE subscriptions_title_group_comments
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move collage_entry (UNIQUE on collage_id, title_group_id)
        sqlx::query!(
            r#"
            DELETE FROM collage_entry
            WHERE title_group_id = $1
              AND collage_id IN (
                  SELECT collage_id FROM collage_entry WHERE title_group_id = $2
              )
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            r#"
            UPDATE collage_entry
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move notifications_title_group_comments
        sqlx::query!(
            r#"
            UPDATE notifications_title_group_comments
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Move notifications_artist_title_groups
        sqlx::query!(
            r#"
            UPDATE notifications_artist_title_groups
            SET title_group_id = $2
            WHERE title_group_id = $1
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Merge external_links from source into target (deduplicated)
        sqlx::query!(
            r#"
            UPDATE title_groups
            SET external_links = (
                SELECT ARRAY(
                    SELECT DISTINCT unnest(t.external_links || s.external_links)
                    FROM title_groups t, title_groups s
                    WHERE t.id = $2 AND s.id = $1
                )
            )
            WHERE id = $2
            "#,
            source_title_group_id,
            target_title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Delete the now-empty source title group
        sqlx::query!(
            r#"
            DELETE FROM title_groups WHERE id = $1
            "#,
            source_title_group_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// user counters are not decremented
    pub async fn delete_title_group(&self, title_group_id: i32) -> Result<()> {
        // Check if there are any undeleted torrents linked to this title group
        let has_undeleted_torrents: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM torrents t
                JOIN edition_groups eg ON eg.id = t.edition_group_id
                WHERE eg.title_group_id = $1 AND t.deleted_at IS NULL
            ) AS "exists!"
            "#,
            title_group_id
        )
        .fetch_one(&self.pool)
        .await?;

        if has_undeleted_torrents {
            return Err(Error::TitleGroupHasUndeletedTorrents);
        }

        // Decrement counters for all affiliated artists before cascade delete
        sqlx::query!(
            r#"
            UPDATE artists
            SET
                title_groups_amount = title_groups_amount - 1,
                edition_groups_amount = edition_groups_amount - (
                    SELECT COUNT(*) FROM edition_groups WHERE title_group_id = $1
                ),
                torrents_amount = torrents_amount - (
                    SELECT COUNT(*) FROM torrents t
                    JOIN edition_groups eg ON eg.id = t.edition_group_id
                    WHERE eg.title_group_id = $1
                )
            WHERE id IN (
                SELECT artist_id FROM affiliated_artists WHERE title_group_id = $1
            )
            "#,
            title_group_id
        )
        .execute(&self.pool)
        .await?;

        // Delete the title group (cascades to edition_groups, affiliated_artists, etc.)
        sqlx::query!(
            r#"
            DELETE FROM title_groups WHERE id = $1
            "#,
            title_group_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
