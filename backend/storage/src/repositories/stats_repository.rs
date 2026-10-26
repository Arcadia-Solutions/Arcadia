use crate::{connection_pool::ConnectionPool, models::home_stats::HomeStats};
use arcadia_common::error::Result;

impl ConnectionPool {
    pub async fn find_home_stats(&self) -> Result<HomeStats> {
        let stats = sqlx::query_as!(
            HomeStats,
            r#"
            SELECT
                (SELECT COUNT(*) FROM users WHERE banned = FALSE)::BIGINT AS "enabled_users!",
                (SELECT COUNT(*) FROM users WHERE last_seen >= NOW() - INTERVAL '1 day')::BIGINT AS "users_active_today!",
                (SELECT COUNT(*) FROM users WHERE last_seen >= NOW() - INTERVAL '7 days')::BIGINT AS "users_active_this_week!",
                (SELECT COUNT(*) FROM users WHERE last_seen >= NOW() - INTERVAL '30 days')::BIGINT AS "users_active_this_month!",
                (SELECT COUNT(*) FROM torrents WHERE deleted_at IS NULL)::BIGINT AS "torrents!",
                (SELECT COUNT(DISTINCT eg.title_group_id) FROM edition_groups eg INNER JOIN torrents t ON t.edition_group_id = eg.id)::BIGINT AS "titles!",
                (SELECT COUNT(*) FROM artists)::BIGINT AS "artists!",
                (SELECT COUNT(*) FROM peers WHERE active = TRUE)::BIGINT AS "peers!",
                (SELECT COUNT(*) FROM peers WHERE active = TRUE AND seeder = TRUE)::BIGINT AS "seeders!",
                (SELECT COUNT(*) FROM peers WHERE active = TRUE AND seeder = FALSE)::BIGINT AS "leechers!",
                (SELECT COUNT(*) FROM torrent_requests)::BIGINT AS "torrent_requests!",
                (SELECT COUNT(*) FROM torrent_requests WHERE filled_by_torrent_id IS NOT NULL)::BIGINT AS "torrent_requests_filled!",
                (SELECT COUNT(*) FROM torrent_activities WHERE completed_at IS NOT NULL)::BIGINT AS "snatches!",
                (SELECT COUNT(*) FROM series)::BIGINT AS "series!",
                (SELECT COUNT(*) FROM collage)::BIGINT AS "collages!",
                (SELECT COUNT(*) FROM forum_threads)::BIGINT AS "forum_threads!",
                (SELECT COUNT(*) FROM forum_posts)::BIGINT AS "forum_posts!",
                (SELECT COUNT(*) FROM title_group_comments)::BIGINT AS "title_group_comments!",
                (SELECT COUNT(*) FROM torrent_request_comments)::BIGINT AS "torrent_request_comments!"
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }
}
