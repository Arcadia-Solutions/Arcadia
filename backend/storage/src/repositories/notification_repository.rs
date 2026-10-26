use crate::{
    connection_pool::ConnectionPool,
    models::{
        notification::{
            NotificationAnnounceError, NotificationArtistTitleGroup, NotificationCollage,
            NotificationCounts, NotificationForumSubCategoryThread, NotificationForumThreadPost,
            NotificationReseedRequest, NotificationStaffPmMessage, NotificationTitleGroupComment,
            NotificationTitleGroupTorrent, NotificationTorrentDeletion,
            NotificationTorrentRequestComment, Notifications,
        },
        torrent::TorrentDeletionReason,
        user::UserLite,
    },
};
use arcadia_common::error::{Error, Result};
use arcadia_shared::tracker::models::announce_error_update::AnnounceErrorCode;
use sqlx::{PgPool, Postgres, Transaction};
use std::borrow::Borrow;

impl ConnectionPool {
    pub async fn find_all_notifications(
        &self,
        user_id: i32,
        include_read: bool,
    ) -> Result<Notifications> {
        let forum_sub_category_threads = sqlx::query_as!(
            NotificationForumSubCategoryThread,
            r#"
            SELECT
                n.id,
                n.forum_thread_id,
                ft.name AS forum_thread_name,
                n.forum_sub_category_id,
                fsc.name AS forum_sub_category_name,
                n.created_at,
                n.read_status
            FROM notifications_forum_sub_category_threads n
            JOIN forum_threads ft ON ft.id = n.forum_thread_id
            JOIN forum_sub_categories fsc ON fsc.id = n.forum_sub_category_id
            WHERE n.user_id = $1
            AND ($2::bool = TRUE OR n.read_status = FALSE)
            ORDER BY n.created_at DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        let forum_thread_posts = sqlx::query_as!(
            NotificationForumThreadPost,
            r#"
            SELECT
                n.id,
                n.forum_post_id,
                p.forum_thread_id,
                t.name AS forum_thread_name,
                n.created_at,
                n.read_status
            FROM notifications_forum_thread_posts n
            JOIN forum_posts p ON p.id = n.forum_post_id
            JOIN forum_threads t ON t.id = n.forum_thread_id
            WHERE n.user_id = $1
            AND ($2::bool = TRUE OR n.read_status = FALSE)
            ORDER BY n.created_at DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        let title_group_comments = sqlx::query_as!(
            NotificationTitleGroupComment,
            r#"
            SELECT
                n.id,
                n.title_group_comment_id,
                n.title_group_id,
                tg.name AS title_group_name,
                n.created_at,
                n.read_status
            FROM notifications_title_group_comments n
            JOIN title_groups tg ON tg.id = n.title_group_id
            WHERE n.user_id = $1
            AND ($2::bool = TRUE OR n.read_status = FALSE)
            ORDER BY n.created_at DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        let title_group_torrents = sqlx::query_as!(
            NotificationTitleGroupTorrent,
            r#"
            SELECT
                n.id,
                n.torrent_id,
                eg.title_group_id,
                tg.name AS title_group_name,
                n.created_at,
                n.read_status
            FROM notifications_title_group_torrents n
            JOIN torrents t ON t.id = n.torrent_id
            JOIN edition_groups eg ON eg.id = t.edition_group_id
            JOIN title_groups tg ON tg.id = eg.title_group_id
            WHERE n.user_id = $1
            AND t.deleted_at IS NULL
            AND ($2::bool = TRUE OR n.read_status = FALSE)
            ORDER BY n.created_at DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        let artist_title_groups = sqlx::query_as!(
            NotificationArtistTitleGroup,
            r#"
            SELECT
                n.id,
                n.artist_id,
                a.name AS artist_name,
                n.title_group_id,
                tg.name AS title_group_name,
                n.created_at,
                n.read_status
            FROM notifications_artist_title_groups n
            JOIN artists a ON a.id = n.artist_id
            JOIN title_groups tg ON tg.id = n.title_group_id
            WHERE n.user_id = $1
            AND ($2::bool = TRUE OR n.read_status = FALSE)
            ORDER BY n.created_at DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        let collages = sqlx::query_as!(
            NotificationCollage,
            r#"
            SELECT
                n.id,
                n.collage_id,
                c.name AS collage_name,
                n.title_group_id,
                tg.name AS title_group_name,
                n.created_at,
                n.read_status
            FROM notifications_collages n
            JOIN collage c ON c.id = n.collage_id
            JOIN title_groups tg ON tg.id = n.title_group_id
            WHERE n.user_id = $1
            AND ($2::bool = TRUE OR n.read_status = FALSE)
            ORDER BY n.created_at DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;
        let torrent_request_comments = sqlx::query_as!(
            NotificationTorrentRequestComment,
            r#"
            SELECT
                n.id,
                n.torrent_request_comment_id,
                n.torrent_request_id,
                tg.name AS title_group_name,
                n.created_at,
                n.read_status
            FROM notifications_torrent_request_comments n
            JOIN torrent_requests tr ON tr.id = n.torrent_request_id
            JOIN title_groups tg ON tg.id = tr.title_group_id
            WHERE n.user_id = $1
            AND ($2::bool = TRUE OR n.read_status = FALSE)
            ORDER BY n.created_at DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        let staff_pm_messages = sqlx::query_as!(
            NotificationStaffPmMessage,
            r#"
            SELECT
                n.id,
                n.staff_pm_message_id,
                n.staff_pm_id,
                sp.subject AS staff_pm_subject,
                n.created_at,
                n.read_status
            FROM notifications_staff_pm_messages n
            JOIN staff_pms sp ON sp.id = n.staff_pm_id
            WHERE n.user_id = $1
            AND ($2::bool = TRUE OR n.read_status = FALSE)
            ORDER BY n.created_at DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        let torrent_deletions = sqlx::query_as!(
            NotificationTorrentDeletion,
            r#"
            SELECT
                td.torrent_id,
                td.title_group_name,
                td.deletion_reason AS "deletion_reason!: TorrentDeletionReason",
                td.extra_information,
                td.replacement_torrent_id,
                td.deleted_at,
                tdn.read_status
            FROM torrent_deletion_notifications tdn
            JOIN torrent_deletions td ON td.torrent_id = tdn.torrent_id
            WHERE tdn.user_id = $1
            AND ($2::bool = TRUE OR tdn.read_status = FALSE)
            ORDER BY td.deleted_at DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        // announce errors have no read status, they are removed once resolved or stale
        let announce_errors = sqlx::query_as!(
            NotificationAnnounceError,
            r#"
            SELECT
                ae.error_code AS "error_code: AnnounceErrorCode",
                encode(ae.info_hash, 'hex') AS "info_hash!",
                ae.torrent_id,
                tg.id AS "title_group_id?",
                tg.name AS "title_group_name?",
                ae.occurrences,
                ae.first_seen_at,
                ae.last_seen_at
            FROM announce_errors ae
            LEFT JOIN torrents t ON t.id = ae.torrent_id
            LEFT JOIN edition_groups eg ON eg.id = t.edition_group_id
            LEFT JOIN title_groups tg ON tg.id = eg.title_group_id
            WHERE ae.user_id = $1
            ORDER BY ae.last_seen_at DESC
            "#,
            user_id
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        let reseed_requests = sqlx::query!(
            r#"
            SELECT
                n.torrent_id AS "torrent_id!",
                eg.title_group_id AS "title_group_id!",
                tg.name AS "title_group_name!",
                MAX(n.created_at) AS "created_at!",
                bool_and(n.read_status) AS "read_status!",
                jsonb_agg(DISTINCT jsonb_build_object(
                    'id', ru.id, 'username', ru.username, 'warned', ru.warned, 'banned', ru.banned
                )) AS "requested_by!: sqlx::types::Json<Vec<UserLite>>"
            FROM notifications_reseed_requests n
            JOIN torrents t ON t.id = n.torrent_id
            JOIN edition_groups eg ON eg.id = t.edition_group_id
            JOIN title_groups tg ON tg.id = eg.title_group_id
            JOIN users ru ON ru.id = n.requested_by_id
            WHERE n.user_id = $1
            AND t.deleted_at IS NULL
            AND ($2::bool = TRUE OR n.read_status = FALSE)
            GROUP BY n.torrent_id, eg.title_group_id, tg.name
            ORDER BY MAX(n.created_at) DESC
            "#,
            user_id,
            include_read
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?
        .into_iter()
        .map(|row| NotificationReseedRequest {
            torrent_id: row.torrent_id,
            title_group_id: row.title_group_id,
            title_group_name: row.title_group_name,
            requested_by: row.requested_by.0,
            created_at: row.created_at,
            read_status: row.read_status,
        })
        .collect();

        Ok(Notifications {
            announce_errors,
            reseed_requests,
            artist_title_groups,
            collages,
            forum_sub_category_threads,
            forum_thread_posts,
            staff_pm_messages,
            title_group_comments,
            title_group_torrents,
            torrent_deletions,
            torrent_request_comments,
        })
    }

    /// Removes the reseed requests of the torrents that are healthy again
    /// (at least one seeder and no leecher waiting), as well as the ones belonging to
    /// soft-deleted torrents (the foreign-key cascade never fires for a soft delete, so
    /// these rows would otherwise accumulate forever).
    pub async fn remove_resolved_reseed_requests(&self) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM notifications_reseed_requests
            WHERE torrent_id IN (
                SELECT id FROM torrents
                WHERE (seeders >= 1 AND leechers = 0) OR deleted_at IS NOT NULL
            )
            "#,
        )
        .execute(self.borrow())
        .await?;

        Ok(result.rows_affected())
    }

    /// Removes the announce errors whose peer announced successfully since then,
    /// and the ones that were not seen again for `retention_seconds`
    pub async fn remove_resolved_and_stale_announce_errors(
        &self,
        retention_seconds: i64,
    ) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM announce_errors ae
            WHERE ae.last_seen_at < NOW() - make_interval(secs => $1::bigint)
            OR EXISTS (
                SELECT 1 FROM peers p
                WHERE p.user_id = ae.user_id
                AND p.torrent_id = ae.torrent_id
                AND p.peer_id = ae.peer_id
                AND p.updated_at > (ae.last_seen_at AT TIME ZONE 'UTC')
            )
            "#,
            retention_seconds
        )
        .execute(self.borrow())
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn record_torrent_deletion(
        tx: &mut Transaction<'_, Postgres>,
        torrent_id: i32,
        title_group_name: &str,
        deletion_reason: TorrentDeletionReason,
        extra_information: Option<&str>,
        replacement_torrent_id: Option<i32>,
        deleted_by_id: i32,
    ) -> Result<Vec<i32>> {
        sqlx::query!(
            r#"
            INSERT INTO torrent_deletions
                (torrent_id, title_group_name, deletion_reason, extra_information, replacement_torrent_id, deleted_by_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            torrent_id,
            title_group_name,
            deletion_reason as TorrentDeletionReason,
            extra_information,
            replacement_torrent_id,
            deleted_by_id
        )
        .execute(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        let user_ids = sqlx::query_scalar!(
            r#"
            WITH recipients AS (
                SELECT DISTINCT user_id
                FROM (
                    SELECT user_id FROM peers WHERE torrent_id = $1
                    UNION
                    SELECT created_by_id AS user_id FROM torrents WHERE id = $1
                ) AS combined
                WHERE user_id IS NOT NULL AND user_id != $2
            )
            INSERT INTO torrent_deletion_notifications (user_id, torrent_id)
            SELECT user_id, $1 FROM recipients
            ON CONFLICT (user_id, torrent_id) DO NOTHING
            RETURNING user_id
            "#,
            torrent_id,
            deleted_by_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        Ok(user_ids)
    }

    pub async fn mark_notifications_torrent_deletions_as_read(
        &self,
        user_id: i32,
        torrent_ids: &[i32],
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE torrent_deletion_notifications
            SET read_status = TRUE
            WHERE user_id = $1 AND torrent_id = ANY($2)
            "#,
            user_id,
            torrent_ids
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;

        Ok(())
    }

    /// Marks every reseed request addressed to `user_id` for the given torrent as
    /// read. The rows are kept (the periodic job removes them once the torrent is healthy again),
    /// they are simply no longer surfaced to the recipient.
    pub async fn mark_notifications_reseed_requests_as_read(
        &self,
        user_id: i32,
        torrent_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE notifications_reseed_requests
            SET read_status = TRUE
            WHERE user_id = $1 AND torrent_id = $2
            "#,
            user_id,
            torrent_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;

        Ok(())
    }

    pub async fn notify_users_title_group_torrents(
        tx: &mut Transaction<'_, Postgres>,
        title_group_id: i32,
        torrent_id: i32,
        current_user_id: i32,
    ) -> Result<Vec<i32>> {
        let user_ids = sqlx::query_scalar!(
            r#"
                WITH user_ids AS (
                    SELECT user_id
                    FROM subscriptions_title_group_torrents
                    WHERE title_group_id = $1
                    AND user_id != $3
                )
                INSERT INTO notifications_title_group_torrents (user_id, torrent_id)
                SELECT
                    user_id,
                    $2
                FROM user_ids
                RETURNING user_id
            "#,
            title_group_id,
            torrent_id,
            current_user_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        Ok(user_ids)
    }

    /// Checks that the torrent is eligible for a reseed request (no seeders, and the last
    /// seeding activity is older than `threshold_hours`), then notifies the past seeders and
    /// snatchers. Returns the ids of the notified users.
    pub async fn request_reseed(
        &self,
        torrent_id: i32,
        requested_by_id: i32,
        threshold_hours: i32,
    ) -> Result<Vec<i32>> {
        let mut tx: Transaction<'_, Postgres> = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        let row = sqlx::query!(
            r#"
                SELECT
                    t.seeders,
                    (SELECT MAX(ta.last_seen_seeding_at)
                     FROM torrent_activities ta
                     WHERE ta.torrent_id = t.id
                    ) <= NOW() - make_interval(hours => $2) AS "dead_long_enough?",
                    EXISTS(
                        SELECT 1 FROM notifications_reseed_requests nrr
                        WHERE nrr.torrent_id = t.id AND nrr.requested_by_id = $3
                    ) AS "already_requested!",
                    EXISTS(
                        SELECT 1 FROM torrent_activities ta
                        WHERE ta.torrent_id = t.id AND ta.user_id <> $3
                          AND (ta.first_seen_seeding_at IS NOT NULL OR ta.completed_at IS NOT NULL)
                    ) AS "has_other_contributors!"
                FROM torrents t
                WHERE t.id = $1 AND t.deleted_at IS NULL
            "#,
            torrent_id,
            threshold_hours,
            requested_by_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(Error::TorrentNotFound)?;

        if row.already_requested {
            return Err(Error::ReseedAlreadyRequested);
        }
        if row.seeders != 0 {
            return Err(Error::ReseedNotEligible(
                "the torrent still has seeders".into(),
            ));
        }
        match row.dead_long_enough {
            None => {
                return Err(Error::ReseedNotEligible(
                    "the torrent has never been seeded".into(),
                ))
            }
            Some(false) => {
                return Err(Error::ReseedNotEligible(format!(
                    "the torrent must have had no seeders for at least {threshold_hours} hours"
                )))
            }
            Some(true) => {}
        }
        if !row.has_other_contributors {
            return Err(Error::ReseedNotEligible(
                "you were the only seeder or snatcher of this torrent".into(),
            ));
        }

        let user_ids =
            Self::notify_users_reseed_request(&mut tx, torrent_id, requested_by_id).await?;
        tx.commit().await?;

        Ok(user_ids)
    }

    /// Records the requester against everyone who has seeded or completed the torrent (except the
    /// requester). A recipient sees a single notification per torrent that aggregates every
    /// requester, so re-requests by the same requester are ignored and additional requesters are
    /// merged into the existing line. Returns only the recipients for whom this created a
    /// brand-new notification line (so only they are notified over SSE).
    pub async fn notify_users_reseed_request(
        tx: &mut Transaction<'_, Postgres>,
        torrent_id: i32,
        requested_by_id: i32,
    ) -> Result<Vec<i32>> {
        // Recipients who do not yet have any reseed notification for this torrent: they get a
        // brand-new line, so they are the ones to notify over SSE.
        let new_recipients = sqlx::query_scalar!(
            r#"
                SELECT ta.user_id AS "user_id!"
                FROM torrent_activities ta
                WHERE ta.torrent_id = $1
                  AND ta.user_id <> $2
                  AND (ta.first_seen_seeding_at IS NOT NULL OR ta.completed_at IS NOT NULL)
                  AND NOT EXISTS (
                      SELECT 1 FROM notifications_reseed_requests n
                      WHERE n.torrent_id = ta.torrent_id
                        AND n.user_id = ta.user_id
                  )
            "#,
            torrent_id,
            requested_by_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        sqlx::query!(
            r#"
                INSERT INTO notifications_reseed_requests (torrent_id, user_id, requested_by_id)
                SELECT ta.torrent_id, ta.user_id, $2
                FROM torrent_activities ta
                WHERE ta.torrent_id = $1
                  AND ta.user_id <> $2
                  AND (ta.first_seen_seeding_at IS NOT NULL OR ta.completed_at IS NOT NULL)
                ON CONFLICT (torrent_id, user_id, requested_by_id) DO NOTHING
            "#,
            torrent_id,
            requested_by_id
        )
        .execute(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        Ok(new_recipients)
    }

    pub async fn notify_users_artist_title_groups(
        tx: &mut Transaction<'_, Postgres>,
        artist_ids: &[i64],
        title_group_id: i32,
        current_user_id: i32,
    ) -> Result<Vec<i32>> {
        let user_ids = sqlx::query_scalar!(
            r#"
                WITH subscribers AS (
                    SELECT user_id, MIN(artist_id) AS artist_id
                    FROM subscriptions_artist_title_groups
                    WHERE artist_id = ANY($1)
                    AND user_id != $3
                    GROUP BY user_id
                )
                INSERT INTO notifications_artist_title_groups (user_id, artist_id, title_group_id)
                SELECT
                    s.user_id,
                    s.artist_id,
                    $2
                FROM subscribers s
                -- only one notification per title group, as long as it stays unread
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM notifications_artist_title_groups n
                    WHERE n.user_id = s.user_id
                      AND n.title_group_id = $2
                      AND n.read_status = FALSE
                )
                RETURNING user_id
            "#,
            artist_ids,
            title_group_id,
            current_user_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        Ok(user_ids)
    }

    pub async fn notify_users_collages(
        tx: &mut Transaction<'_, Postgres>,
        collage_id: i64,
        title_group_id: i32,
        current_user_id: i32,
    ) -> Result<Vec<i32>> {
        let user_ids = sqlx::query_scalar!(
            r#"
                INSERT INTO notifications_collages (
                    user_id,
                    collage_id,
                    title_group_id
                )
                SELECT
                    user_id,
                    $1,
                    $2
                FROM subscriptions_collages
                WHERE collage_id = $1
                  AND user_id != $3
                RETURNING user_id
            "#,
            collage_id,
            title_group_id,
            current_user_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;
        Ok(user_ids)
    }
    pub async fn notify_users_forum_sub_category_threads(
        tx: &mut Transaction<'_, Postgres>,
        forum_sub_category_id: i32,
        thread_id: i64,
        current_user_id: i32,
    ) -> Result<Vec<i32>> {
        let user_ids = sqlx::query_scalar!(
            r#"
                WITH user_ids AS (
                    SELECT user_id
                    FROM subscriptions_forum_sub_category_threads
                    WHERE forum_sub_category_id = $1
                    AND user_id != $3
                )
                INSERT INTO notifications_forum_sub_category_threads (user_id, forum_sub_category_id, forum_thread_id)
                SELECT
                    user_id,
                    $1,
                    $2
                FROM user_ids u
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM notifications_forum_sub_category_threads n
                    WHERE n.user_id = u.user_id
                      AND n.forum_sub_category_id = $1
                      AND n.read_status = FALSE
                )
                RETURNING user_id
            "#,
            forum_sub_category_id,
            thread_id,
            current_user_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        Ok(user_ids)
    }

    pub async fn mark_notification_forum_sub_category_thread_as_read_by_thread_id(
        &self,
        forum_thread_id: i64,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE notifications_forum_sub_category_threads AS notification
                SET read_status = TRUE
                WHERE notification.forum_thread_id = $1
                  AND notification.user_id = $2
                  AND EXISTS (
                      SELECT 1 FROM subscriptions_forum_sub_category_threads subscription
                      WHERE subscription.forum_sub_category_id = notification.forum_sub_category_id
                        AND subscription.user_id = $2
                  )
            "#,
            forum_thread_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;

        Ok(())
    }

    pub async fn notify_users_forum_thread_posts(
        tx: &mut Transaction<'_, Postgres>,
        thread_id: i64,
        post_id: i64,
        current_user_id: i32,
    ) -> Result<Vec<i32>> {
        let user_ids = sqlx::query_scalar!(
            r#"
                WITH user_ids AS (
                    SELECT user_id
                    FROM subscriptions_forum_thread_posts
                    WHERE forum_thread_id = $1
                    AND user_id != $3
                )
                INSERT INTO notifications_forum_thread_posts (user_id, forum_post_id, forum_thread_id)
                SELECT
                    user_id,
                    $2,
                    $1
                FROM user_ids u
                -- don't notify the user who created the post
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM notifications_forum_thread_posts n
                    WHERE n.user_id = u.user_id
                      AND n.forum_thread_id = $1
                      AND n.read_status = FALSE
                )
                RETURNING user_id
            "#,
            thread_id,
            post_id,
            current_user_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        Ok(user_ids)
    }

    pub async fn mark_notification_title_group_torrent_as_read(
        &self,
        title_group_id: i32,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE notifications_title_group_torrents n
                SET read_status = TRUE
                FROM torrents t
                JOIN edition_groups eg ON eg.id = t.edition_group_id
                WHERE n.torrent_id = t.id
                  AND eg.title_group_id = $1
                  AND n.user_id = $2
            "#,
            title_group_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;

        Ok(())
    }

    pub async fn mark_notification_artist_title_group_as_read(
        &self,
        title_group_id: i32,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE notifications_artist_title_groups
                SET read_status = TRUE
                WHERE title_group_id = $1 AND user_id = $2
            "#,
            title_group_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;

        Ok(())
    }

    pub async fn mark_notification_collage_as_read(
        &self,
        collage_id: i64,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE notifications_collages
                SET read_status = TRUE
                WHERE collage_id = $1 AND user_id = $2
            "#,
            collage_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;
        Ok(())
    }
    pub async fn mark_notification_forum_thread_post_as_read(
        &self,
        forum_thread_id: i64,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE notifications_forum_thread_posts
                SET read_status = TRUE
                WHERE forum_thread_id = $1 AND user_id = $2
            "#,
            forum_thread_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumThread)?;

        Ok(())
    }

    pub async fn notify_users_title_group_comments(
        tx: &mut Transaction<'_, Postgres>,
        title_group_id: i32,
        comment_id: i64,
        current_user_id: i32,
    ) -> Result<Vec<i32>> {
        let user_ids = sqlx::query_scalar!(
            r#"
                WITH user_ids AS (
                    SELECT user_id
                    FROM subscriptions_title_group_comments
                    WHERE title_group_id = $1
                    AND user_id != $3
                )
                INSERT INTO notifications_title_group_comments (user_id, title_group_comment_id, title_group_id)
                SELECT
                    user_id,
                    $2,
                    $1
                FROM user_ids u
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM notifications_title_group_comments n
                    WHERE n.user_id = u.user_id
                      AND n.title_group_id = $1
                      AND n.read_status = FALSE
                )
                RETURNING user_id
            "#,
            title_group_id,
            comment_id,
            current_user_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        Ok(user_ids)
    }

    pub async fn mark_notification_title_group_comment_as_read(
        &self,
        title_group_id: i32,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE notifications_title_group_comments
                SET read_status = TRUE
                WHERE title_group_id = $1 AND user_id = $2
            "#,
            title_group_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;

        Ok(())
    }

    pub async fn notify_users_torrent_request_comments(
        tx: &mut Transaction<'_, Postgres>,
        torrent_request_id: i64,
        comment_id: i64,
        current_user_id: i32,
    ) -> Result<Vec<i32>> {
        let user_ids = sqlx::query_scalar!(
            r#"
                WITH user_ids AS (
                    SELECT user_id
                    FROM subscriptions_torrent_request_comments
                    WHERE torrent_request_id = $1
                    AND user_id != $3
                )
                INSERT INTO notifications_torrent_request_comments (user_id, torrent_request_comment_id, torrent_request_id)
                SELECT
                    user_id,
                    $2,
                    $1
                FROM user_ids u
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM notifications_torrent_request_comments n
                    WHERE n.user_id = u.user_id
                      AND n.torrent_request_id = $1
                      AND n.read_status = FALSE
                )
                RETURNING user_id
            "#,
            torrent_request_id,
            comment_id,
            current_user_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        Ok(user_ids)
    }

    pub async fn mark_notification_torrent_request_comment_as_read(
        &self,
        torrent_request_id: i64,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE notifications_torrent_request_comments
                SET read_status = TRUE
                WHERE torrent_request_id = $1 AND user_id = $2
            "#,
            torrent_request_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;

        Ok(())
    }

    pub async fn notify_users_staff_pm_messages(
        tx: &mut Transaction<'_, Postgres>,
        staff_pm_id: i64,
        staff_pm_message_id: i64,
        current_user_id: i32,
    ) -> Result<Vec<i32>> {
        let user_ids = sqlx::query_scalar!(
            r#"
                WITH eligible_users AS (
                    -- Staff PM creator
                    SELECT created_by_id AS user_id
                    FROM staff_pms
                    WHERE id = $1
                    UNION
                    -- Users with read_staff_pm permission
                    SELECT id AS user_id
                    FROM users
                    WHERE 'read_staff_pm' = ANY(permissions)
                )
                INSERT INTO notifications_staff_pm_messages (user_id, staff_pm_id, staff_pm_message_id)
                SELECT
                    user_id,
                    $1,
                    $2
                FROM eligible_users u
                WHERE u.user_id != $3
                AND NOT EXISTS (
                    SELECT 1
                    FROM notifications_staff_pm_messages n
                    WHERE n.user_id = u.user_id
                      AND n.staff_pm_id = $1
                      AND n.read_status = FALSE
                )
                RETURNING user_id
            "#,
            staff_pm_id,
            staff_pm_message_id,
            current_user_id
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(Error::CouldNotCreateNotification)?;

        Ok(user_ids)
    }

    pub async fn mark_notifications_staff_pm_messages_as_read(
        &self,
        staff_pm_id: i64,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE notifications_staff_pm_messages
                SET read_status = TRUE
                WHERE staff_pm_id = $1
            "#,
            staff_pm_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;

        Ok(())
    }

    pub async fn mark_notification_staff_pm_message_as_read(
        &self,
        staff_pm_id: i64,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE notifications_staff_pm_messages
                SET read_status = TRUE
                WHERE staff_pm_id = $1 AND user_id = $2
            "#,
            staff_pm_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotMarkNotificationAsRead)?;

        Ok(())
    }

    pub async fn find_notification_counts(&self, user_id: i32) -> Result<NotificationCounts> {
        let counts = sqlx::query_as!(
            NotificationCounts,
            r#"
            SELECT
                (SELECT COUNT(*)
                 FROM forum_threads ft
                 WHERE ft.forum_sub_category_id = 1
                   AND NOT EXISTS (
                       SELECT 1 FROM forum_thread_reads ftr
                       WHERE ftr.forum_thread_id = ft.id AND ftr.user_id = $1
                   )
                )::int4 AS "announcements!",
                (SELECT COUNT(*)
                 FROM conversations c
                 JOIN LATERAL (
                     SELECT cm.created_at, cm.created_by_id
                     FROM conversation_messages cm
                     WHERE cm.conversation_id = c.id
                     ORDER BY cm.created_at DESC
                     LIMIT 1
                 ) AS lm ON TRUE
                 WHERE lm.created_by_id != $1
                   AND (
                       (c.sender_id = $1 AND c.sender_last_seen_at < lm.created_at)
                       OR
                       (c.receiver_id = $1 AND (c.receiver_last_seen_at IS NULL OR c.receiver_last_seen_at < lm.created_at))
                   )
                )::int4 AS "conversations!",
                (SELECT COUNT(*)
                 FROM notifications_forum_sub_category_threads
                 WHERE user_id = $1 AND read_status = FALSE
                )::int4 AS "forum_sub_category_threads!",
                (SELECT COUNT(*)
                 FROM notifications_forum_thread_posts
                 WHERE user_id = $1 AND read_status = FALSE
                )::int4 AS "forum_thread_posts!",
                (SELECT COUNT(*)
                 FROM notifications_title_group_comments
                 WHERE user_id = $1 AND read_status = FALSE
                )::int4 AS "title_group_comments!",
                (SELECT COUNT(*)
                 FROM notifications_title_group_torrents n
                 JOIN torrents t ON t.id = n.torrent_id
                 WHERE n.user_id = $1 AND n.read_status = FALSE
                   AND t.deleted_at IS NULL
                )::int4 AS "title_group_torrents!",
                (SELECT COUNT(*)
                 FROM notifications_artist_title_groups
                 WHERE user_id = $1 AND read_status = FALSE
                )::int4 AS "artist_title_groups!",
                (SELECT COUNT(*)
                 FROM notifications_collages
                 WHERE user_id = $1 AND read_status = FALSE
                )::int4 AS "collages!",
                (SELECT COUNT(*)
                 FROM notifications_staff_pm_messages
                 WHERE user_id = $1 AND read_status = FALSE
                )::int4 AS "staff_pm_messages!",
                (SELECT COUNT(*)
                 FROM notifications_torrent_request_comments
                 WHERE user_id = $1 AND read_status = FALSE
                )::int4 AS "torrent_request_comments!",
                (SELECT COUNT(*)
                 FROM torrent_deletion_notifications
                 WHERE user_id = $1 AND read_status = FALSE
                )::int4 AS "torrent_deletions!",
                (SELECT COUNT(*)
                 FROM announce_errors
                 WHERE user_id = $1
                )::int4 AS "announce_errors!",
                (SELECT COUNT(DISTINCT n.torrent_id)
                 FROM notifications_reseed_requests n
                 JOIN torrents t ON t.id = n.torrent_id
                 WHERE n.user_id = $1 AND n.read_status = FALSE
                   AND t.deleted_at IS NULL
                )::int4 AS "reseed_requests!"
            "#,
            user_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotGetUnreadNotifications)?;

        Ok(counts)
    }
}
