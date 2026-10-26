use crate::{
    connection_pool::ConnectionPool,
    models::forum_stats::{
        ForumStatsDataPoint, ForumStatsMetric, ForumStatsQuery, ForumStatsResponse,
    },
};
use arcadia_common::error::Result;

impl ConnectionPool {
    /// Returns forum activity statistics (threads or posts) over the requested period,
    /// grouped by the requested dimension (category, sub-category, thread, user, user class, or none).
    /// Periods with no data are filled with zero values, starting from the earliest forum activity.
    pub async fn get_forum_stats(&self, query: &ForumStatsQuery) -> Result<ForumStatsResponse> {
        let data = match query.metric {
            ForumStatsMetric::Posts => {
                sqlx::query_as!(
                    ForumStatsDataPoint,
                    r#"
                    WITH periods AS (
                        SELECT generate_series(
                            date_trunc($3, $1::DATE::TIMESTAMP),
                            date_trunc($3, $2::DATE::TIMESTAMP),
                            ('1 ' || $3)::INTERVAL
                        ) AS period
                    ),
                    stats AS (
                        SELECT
                            date_trunc($3, fp.created_at)::TIMESTAMP AS period,
                            COUNT(*)::BIGINT AS count,
                            COALESCE(SUM(LENGTH(fp.content)), 0)::BIGINT AS total_content_length,
                            CASE $4
                                WHEN 'category' THEN fc.name
                                WHEN 'sub_category' THEN fsc.name
                                WHEN 'thread' THEN ft.name
                                WHEN 'user' THEN u.username
                                WHEN 'user_class' THEN u.class_name
                                ELSE NULL
                            END AS attribute_value
                        FROM forum_posts fp
                        JOIN forum_threads ft ON fp.forum_thread_id = ft.id
                        JOIN forum_sub_categories fsc ON ft.forum_sub_category_id = fsc.id
                        JOIN forum_categories fc ON fsc.forum_category_id = fc.id
                        JOIN users u ON fp.created_by_id = u.id
                        WHERE fp.created_at >= $1::DATE
                          AND fp.created_at < ($2::DATE + INTERVAL '1 day')
                        GROUP BY period, attribute_value
                    ),
                    attribute_values AS (
                        SELECT DISTINCT attribute_value FROM stats WHERE attribute_value IS NOT NULL
                    )
                    SELECT
                        p.period::TIMESTAMP AS "period!",
                        COALESCE(s.count, 0)::BIGINT AS "count!",
                        COALESCE(s.total_content_length, 0)::BIGINT AS "total_content_length!",
                        av.attribute_value
                    FROM periods p
                    LEFT JOIN attribute_values av ON TRUE
                    LEFT JOIN stats s ON p.period = s.period
                        AND s.attribute_value IS NOT DISTINCT FROM av.attribute_value
                    WHERE p.period >= COALESCE(
                        date_trunc(
                            $3,
                            (SELECT MIN(created_at) FROM forum_posts)
                        )::TIMESTAMP,
                        p.period
                    )
                    ORDER BY p.period, av.attribute_value
                    "#,
                    query.from,
                    query.to,
                    &query.interval.to_string(),
                    &query.group_by.to_string(),
                )
                .fetch_all(&self.pool)
                .await?
            }
            ForumStatsMetric::Threads => {
                sqlx::query_as!(
                    ForumStatsDataPoint,
                    r#"
                    WITH periods AS (
                        SELECT generate_series(
                            date_trunc($3, $1::DATE::TIMESTAMP),
                            date_trunc($3, $2::DATE::TIMESTAMP),
                            ('1 ' || $3)::INTERVAL
                        ) AS period
                    ),
                    stats AS (
                        SELECT
                            date_trunc($3, ft.created_at)::TIMESTAMP AS period,
                            COUNT(*)::BIGINT AS count,
                            COALESCE(SUM(LENGTH(ft.name)), 0)::BIGINT AS total_content_length,
                            CASE $4
                                WHEN 'category' THEN fc.name
                                WHEN 'sub_category' THEN fsc.name
                                WHEN 'thread' THEN ft.name
                                WHEN 'user' THEN u.username
                                WHEN 'user_class' THEN u.class_name
                                ELSE NULL
                            END AS attribute_value
                        FROM forum_threads ft
                        JOIN forum_sub_categories fsc ON ft.forum_sub_category_id = fsc.id
                        JOIN forum_categories fc ON fsc.forum_category_id = fc.id
                        JOIN users u ON ft.created_by_id = u.id
                        WHERE ft.created_at >= $1::DATE
                          AND ft.created_at < ($2::DATE + INTERVAL '1 day')
                        GROUP BY period, attribute_value
                    ),
                    attribute_values AS (
                        SELECT DISTINCT attribute_value FROM stats WHERE attribute_value IS NOT NULL
                    )
                    SELECT
                        p.period::TIMESTAMP AS "period!",
                        COALESCE(s.count, 0)::BIGINT AS "count!",
                        COALESCE(s.total_content_length, 0)::BIGINT AS "total_content_length!",
                        av.attribute_value
                    FROM periods p
                    LEFT JOIN attribute_values av ON TRUE
                    LEFT JOIN stats s ON p.period = s.period
                        AND s.attribute_value IS NOT DISTINCT FROM av.attribute_value
                    WHERE p.period >= COALESCE(
                        date_trunc(
                            $3,
                            (SELECT MIN(created_at) FROM forum_threads)
                        )::TIMESTAMP,
                        p.period
                    )
                    ORDER BY p.period, av.attribute_value
                    "#,
                    query.from,
                    query.to,
                    &query.interval.to_string(),
                    &query.group_by.to_string(),
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        let summary = sqlx::query!(
            r#"
            SELECT
                p.unique_posters AS "unique_posters!",
                p.total_posts_created AS "total_posts_created!",
                p.total_content_length AS "total_content_length!",
                t.unique_thread_creators AS "unique_thread_creators!",
                t.total_threads_created AS "total_threads_created!"
            FROM
                (SELECT
                    COUNT(DISTINCT created_by_id)::BIGINT AS unique_posters,
                    COUNT(*)::BIGINT AS total_posts_created,
                    COALESCE(SUM(LENGTH(content)), 0)::BIGINT AS total_content_length
                 FROM forum_posts
                 WHERE created_at >= $1::DATE
                   AND created_at < ($2::DATE + INTERVAL '1 day')
                ) p,
                (SELECT
                    COUNT(DISTINCT created_by_id)::BIGINT AS unique_thread_creators,
                    COUNT(*)::BIGINT AS total_threads_created
                 FROM forum_threads
                 WHERE created_at >= $1::DATE
                   AND created_at < ($2::DATE + INTERVAL '1 day')
                ) t
            "#,
            query.from,
            query.to,
        )
        .fetch_one(&self.pool)
        .await?;

        let average_posts_per_thread = if summary.total_threads_created > 0 {
            summary.total_posts_created as f64 / summary.total_threads_created as f64
        } else {
            0.0
        };
        let average_post_length = if summary.total_posts_created > 0 {
            summary.total_content_length as f64 / summary.total_posts_created as f64
        } else {
            0.0
        };

        Ok(ForumStatsResponse {
            unique_posters: summary.unique_posters,
            unique_thread_creators: summary.unique_thread_creators,
            total_threads_created: summary.total_threads_created,
            total_posts_created: summary.total_posts_created,
            total_content_length: summary.total_content_length,
            average_posts_per_thread,
            average_post_length,
            data,
        })
    }
}
