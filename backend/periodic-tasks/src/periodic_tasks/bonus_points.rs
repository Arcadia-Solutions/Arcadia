use arcadia_storage::connection_pool::ConnectionPool;
use sqlx::PgPool;
use std::borrow::Borrow;
use std::sync::Arc;

pub async fn update_seedtime_and_bonus_points(
    pool: Arc<ConnectionPool>,
    increment_seconds: u64,
    formula_sql: String,
) -> Result<u64, sqlx::Error> {
    let mut transaction = <ConnectionPool as Borrow<PgPool>>::borrow(&pool)
        .begin()
        .await?;

    let seedtime_result = sqlx::query!(
        r#"
        UPDATE torrent_activities ta
        SET total_seed_time = total_seed_time + $1
        WHERE EXISTS (
            SELECT 1 FROM peers p
            WHERE p.torrent_id = ta.torrent_id
              AND p.user_id = ta.user_id
              AND p.seeder = true
              AND p.active = true
        )
        "#,
        increment_seconds as i64
    )
    .execute(&mut *transaction)
    .await?;

    let bonus_query = format!(
        r#"
        WITH activity_bonus AS (
            SELECT
                ta.id AS activity_id,
                ta.user_id,
                t.size AS torrent_size,
                ROUND({formula})::bigint * CASE
                    WHEN (SELECT reward_bonus_points_per_seeding_client FROM arcadia_settings LIMIT 1)
                    THEN seeding_clients.active_seeding_clients
                    ELSE 1
                END AS bonus
            FROM torrent_activities ta
            -- only torrents checked by staff reward bonus points
            INNER JOIN torrents t ON ta.torrent_id = t.id AND t.staff_checked
            INNER JOIN LATERAL (
                SELECT COUNT(*) AS active_seeding_clients
                FROM peers p
                WHERE p.torrent_id = ta.torrent_id
                  AND p.user_id = ta.user_id
                  AND p.seeder = true
                  AND p.active = true
            ) seeding_clients ON seeding_clients.active_seeding_clients > 0
        ),
        update_activities AS (
            UPDATE torrent_activities ta
            SET bonus_points = ta.bonus_points + ab.bonus
            FROM activity_bonus ab
            WHERE ta.id = ab.activity_id AND ab.bonus > 0
        ),
        user_totals AS (
            SELECT
                user_id,
                SUM(bonus)::bigint AS total_bonus,
                COUNT(*)::bigint AS seeded_torrents,
                SUM(torrent_size)::bigint AS total_size
            FROM activity_bonus
            WHERE bonus > 0
            GROUP BY user_id
        ),
        updated_users AS (
            UPDATE users u
            SET bonus_points = u.bonus_points + ut.total_bonus
            FROM user_totals ut
            WHERE u.id = ut.user_id
        )
        INSERT INTO bonus_points_logs (user_id, action, amount, details)
        SELECT
            user_id,
            'seedtime_reward'::bonus_points_log_action_enum,
            total_bonus,
            FORMAT('seeding %s torrents, seeding size %s', seeded_torrents, pg_size_pretty(total_size))
        FROM user_totals
        "#,
        formula = formula_sql
    );

    let bonus_result = sqlx::query(sqlx::AssertSqlSafe(bonus_query))
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    let seedtime_count = seedtime_result.rows_affected();
    let bonus_count = bonus_result.rows_affected();
    log::info!(
        "Updated seedtime for {} torrent activities (+{}s), bonus points for {} users",
        seedtime_count,
        increment_seconds,
        bonus_count
    );
    Ok(seedtime_count + bonus_count)
}
