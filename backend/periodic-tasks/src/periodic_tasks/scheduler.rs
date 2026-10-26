use std::future::Future;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

use arcadia_shared::telemetry::{instrument_periodic_task, PeriodicTaskInstruments};
use arcadia_storage::connection_pool::ConnectionPool;

use crate::config::{formula_to_sql, PeriodicTasksConfig};

use super::bonus_points::update_seedtime_and_bonus_points;
use super::inactive_users::ban_inactive_users;
use super::materialized_views::refresh_title_group_hierarchy_lite;
use super::seeding_size::update_user_torrent_stats;
use super::user_badges::evaluate_user_badges;
use super::user_classes::process_user_class_changes;

static INSTRUMENTS: OnceLock<PeriodicTaskInstruments> = OnceLock::new();

fn instruments() -> &'static PeriodicTaskInstruments {
    INSTRUMENTS
        .get_or_init(|| PeriodicTaskInstruments::register("arcadia.backend", "periodic_task"))
}

/// A job running `task` every `interval_seconds`, instrumented under `task_name`.
fn repeated_job<Task, TaskFuture, TaskError>(
    pool: &Arc<ConnectionPool>,
    interval_seconds: u64,
    task_name: &'static str,
    task: Task,
) -> Result<Job, JobSchedulerError>
where
    Task: Fn(Arc<ConnectionPool>) -> TaskFuture + Send + Sync + 'static,
    TaskFuture: Future<Output = Result<u64, TaskError>> + Send + 'static,
    TaskError: std::fmt::Display + 'static,
{
    let pool = Arc::clone(pool);
    Job::new_repeated_async(
        Duration::from_secs(interval_seconds),
        move |_job_id, _job_scheduler| {
            let task_future = task(Arc::clone(&pool));
            Box::pin(instrument_periodic_task(
                instruments(),
                task_name,
                move || task_future,
            ))
        },
    )
}

/// The pool and the configuration are the ones of the service running the tasks.
pub async fn run_periodic_tasks(
    pool: Arc<ConnectionPool>,
    config: PeriodicTasksConfig,
) -> Result<JobScheduler, Box<dyn std::error::Error>> {
    let scheduler = JobScheduler::new().await?;
    let _ = instruments();

    scheduler
        .add(repeated_job(
            &pool,
            config.user_class_changes_seconds,
            "user_class_changes",
            process_user_class_changes,
        )?)
        .await?;

    let seedtime_and_bonus_points_interval_seconds =
        config.seedtime_and_bonus_points_update_seconds;
    let bonus_formula_sql =
        formula_to_sql(&config.bonus_points_formula, "t.seeders").expect("invalid bonus formula");
    scheduler
        .add(repeated_job(
            &pool,
            seedtime_and_bonus_points_interval_seconds,
            "seedtime_and_bonus_points",
            move |pool| {
                update_seedtime_and_bonus_points(
                    pool,
                    seedtime_and_bonus_points_interval_seconds,
                    bonus_formula_sql.clone(),
                )
            },
        )?)
        .await?;

    scheduler
        .add(repeated_job(
            &pool,
            config.user_torrent_stats_update_seconds,
            "user_torrent_stats",
            update_user_torrent_stats,
        )?)
        .await?;

    scheduler
        .add(repeated_job(
            &pool,
            config.inactive_user_ban_seconds,
            "inactive_user_ban",
            ban_inactive_users,
        )?)
        .await?;

    scheduler
        .add(repeated_job(
            &pool,
            config.expired_warnings_seconds,
            "expired_warnings",
            |pool| async move { pool.clear_expired_warnings().await },
        )?)
        .await?;

    // torrent-level peers (seeders, leechers, times_completed) are updated by the tracker itself,
    // this only aggregates them up to the artist level
    scheduler
        .add(repeated_job(
            &pool,
            config.artist_peer_stats_update_seconds,
            "artist_peer_stats",
            |pool| async move { pool.update_artist_peer_stats().await },
        )?)
        .await?;

    let mut materialized_view_refresh_job = repeated_job(
        &pool,
        config.materialized_view_refresh_seconds,
        "materialized_view_refresh",
        refresh_title_group_hierarchy_lite,
    )?;
    // first tick on startup instead of one interval after it
    let mut materialized_view_refresh_job_data = materialized_view_refresh_job.job_data()?;
    materialized_view_refresh_job_data.next_tick = chrono::Utc::now().timestamp() as u64;
    materialized_view_refresh_job.set_job_data(materialized_view_refresh_job_data)?;
    scheduler.add(materialized_view_refresh_job).await?;

    scheduler
        .add(repeated_job(
            &pool,
            config.user_badges_evaluation_seconds,
            "user_badges_evaluation",
            evaluate_user_badges,
        )?)
        .await?;

    let announce_errors_retention_seconds = config.announce_errors_retention_seconds as i64;
    scheduler
        .add(repeated_job(
            &pool,
            config.announce_errors_cleanup_seconds,
            "announce_errors_cleanup",
            move |pool| async move {
                pool.remove_resolved_and_stale_announce_errors(announce_errors_retention_seconds)
                    .await
            },
        )?)
        .await?;

    scheduler
        .add(repeated_job(
            &pool,
            config.reseed_request_cleanup_seconds,
            "reseed_request_cleanup",
            |pool| async move { pool.remove_resolved_reseed_requests().await },
        )?)
        .await?;

    scheduler.start().await?;

    Ok(scheduler)
}
