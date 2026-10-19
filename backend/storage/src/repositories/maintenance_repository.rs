use crate::connection_pool::ConnectionPool;
use arcadia_common::error::Result;
use sqlx::PgPool;
use std::borrow::Borrow;

impl ConnectionPool {
    /// Recomputes every "cached SQL COUNT" attribute from its source tables.
    ///
    /// If they drift out of sync (manual database edits for example), this runs
    /// all the recomputation queries in a single transaction.
    pub async fn recompute_cached_amounts(&self) -> Result<()> {
        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        sqlx::query!(
            "UPDATE artists
             SET title_groups_amount = (
                 SELECT COUNT(DISTINCT title_group_id)
                 FROM affiliated_artists
                 WHERE affiliated_artists.artist_id = artists.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE artists
             SET edition_groups_amount = (
                 SELECT COUNT(DISTINCT eg.id)
                 FROM edition_groups eg
                 WHERE eg.title_group_id IN (
                     SELECT DISTINCT title_group_id
                     FROM affiliated_artists
                     WHERE affiliated_artists.artist_id = artists.id
                 )
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE artists
             SET torrents_amount = (
                 SELECT COUNT(DISTINCT t.id)
                 FROM torrents t
                 JOIN edition_groups eg ON t.edition_group_id = eg.id
                 WHERE eg.title_group_id IN (
                     SELECT DISTINCT title_group_id
                     FROM affiliated_artists
                     WHERE affiliated_artists.artist_id = artists.id
                 )
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE artists
             SET total_size = (
                 SELECT COALESCE(SUM(t.size), 0)
                 FROM torrents t
                 JOIN edition_groups eg ON t.edition_group_id = eg.id
                 WHERE t.deleted_at IS NULL
                   AND eg.title_group_id IN (
                     SELECT DISTINCT title_group_id
                     FROM affiliated_artists
                     WHERE affiliated_artists.artist_id = artists.id
                 )
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET forum_posts = (
                 SELECT COUNT(*)
                 FROM forum_posts
                 WHERE forum_posts.created_by_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET forum_threads = (
                 SELECT COUNT(*)
                 FROM forum_threads
                 WHERE forum_threads.created_by_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE forum_threads
             SET posts_amount = (
                 SELECT COUNT(*)
                 FROM forum_posts
                 WHERE forum_posts.forum_thread_id = forum_threads.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE forum_sub_categories
             SET threads_amount = (
                     SELECT COUNT(*)
                     FROM forum_threads
                     WHERE forum_threads.forum_sub_category_id = forum_sub_categories.id
                 ),
                 posts_amount = (
                     SELECT COUNT(*)
                     FROM forum_posts
                     JOIN forum_threads ON forum_posts.forum_thread_id = forum_threads.id
                     WHERE forum_threads.forum_sub_category_id = forum_sub_categories.id
                 )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET title_group_comments = (
                 SELECT COUNT(*)
                 FROM title_group_comments
                 WHERE title_group_comments.created_by_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET request_comments = (
                 SELECT COUNT(*)
                 FROM torrent_request_comments
                 WHERE torrent_request_comments.created_by_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET requests_voted = (
                 SELECT COUNT(DISTINCT torrent_request_id)
                 FROM torrent_request_votes
                 WHERE torrent_request_votes.created_by_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET requests_filled = (
                 SELECT COUNT(*)
                 FROM torrent_requests
                 WHERE torrent_requests.filled_by_user_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET collages_started = (
                 SELECT COUNT(*)
                 FROM collage
                 WHERE collage.created_by_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET title_groups = (
                 SELECT COUNT(*)
                 FROM title_groups
                 WHERE title_groups.created_by_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET edition_groups = (
                 SELECT COUNT(*)
                 FROM edition_groups
                 WHERE edition_groups.created_by_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET torrents = (
                 SELECT COUNT(*)
                 FROM torrents
                 WHERE torrents.created_by_id = users.id
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE users
             SET invited = (
                 SELECT COUNT(*)
                 FROM invitations
                 WHERE invitations.sender_id = users.id
                   AND invitations.receiver_id IS NOT NULL
             )"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE artists
             SET seeders_amount = COALESCE((
                     SELECT SUM(t.seeders)
                     FROM torrents t
                     JOIN edition_groups eg ON t.edition_group_id = eg.id
                     JOIN affiliated_artists aa ON aa.title_group_id = eg.title_group_id
                     WHERE aa.artist_id = artists.id
                       AND t.deleted_at IS NULL
                 ), 0),
                 leechers_amount = COALESCE((
                     SELECT SUM(t.leechers)
                     FROM torrents t
                     JOIN edition_groups eg ON t.edition_group_id = eg.id
                     JOIN affiliated_artists aa ON aa.title_group_id = eg.title_group_id
                     WHERE aa.artist_id = artists.id
                       AND t.deleted_at IS NULL
                 ), 0),
                 snatches_amount = COALESCE((
                     SELECT SUM(t.times_completed)
                     FROM torrents t
                     JOIN edition_groups eg ON t.edition_group_id = eg.id
                     JOIN affiliated_artists aa ON aa.title_group_id = eg.title_group_id
                     WHERE aa.artist_id = artists.id
                       AND t.deleted_at IS NULL
                 ), 0)"
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE torrents
             SET seeders = (
                     SELECT COUNT(*)
                     FROM peers
                     WHERE peers.torrent_id = torrents.id
                       AND peers.seeder = true
                       AND peers.active = true
                 ),
                 leechers = (
                     SELECT COUNT(*)
                     FROM peers
                     WHERE peers.torrent_id = torrents.id
                       AND peers.seeder = false
                       AND peers.active = true
                 )"
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
