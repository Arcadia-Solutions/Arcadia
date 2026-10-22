use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

use arcadia_shared::telemetry::{instrument_periodic_task, PeriodicTaskInstruments};

use crate::store::Store;

use super::announce_errors::remove_resolved_and_stale_announce_errors;
use super::bonus_points::update_seedtime_and_bonus_points;
use super::expired_warnings::clear_expired_warnings;
use super::inactive_users::ban_inactive_users;
use super::materialized_views::refresh_title_group_hierarchy_lite;
use super::peers::update_artist_peer_stats;
use super::reseed_requests::remove_resolved_reseed_requests;
use super::seeding_size::update_user_torrent_stats;
use super::user_badges::evaluate_user_badges;
use super::user_classes::process_user_class_changes;

static INSTRUMENTS: OnceLock<PeriodicTaskInstruments> = OnceLock::new();

fn instruments() -> &'static PeriodicTaskInstruments {
    INSTRUMENTS
        .get_or_init(|| PeriodicTaskInstruments::register("arcadia.backend", "periodic_task"))
}

pub async fn run_periodic_tasks(
    store: Arc<Store>,
) -> Result<JobScheduler, Box<dyn std::error::Error>> {
    let sched = JobScheduler::new().await?;
    let _ = instruments();

    let pool_user_classes = Arc::clone(&store.pool);
    let user_class_job = Job::new_repeated_async(
        Duration::from_secs(store.config.user_class_changes_seconds),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_user_classes);
            Box::pin(instrument_periodic_task(
                instruments(),
                "user_class_changes",
                move || process_user_class_changes(pool),
            ))
        },
    )?;
    sched.add(user_class_job).await?;

    let pool_seedtime = Arc::clone(&store.pool);
    let seedtime_and_bonus_points_interval_secs =
        store.config.seedtime_and_bonus_points_update_seconds;
    let bonus_formula_sql = store.config.bonus_points_formula.clone();
    let seedtime_and_bonus_job = Job::new_repeated_async(
        Duration::from_secs(seedtime_and_bonus_points_interval_secs),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_seedtime);
            let formula_sql = bonus_formula_sql.clone();
            Box::pin(instrument_periodic_task(
                instruments(),
                "seedtime_and_bonus_points",
                move || {
                    update_seedtime_and_bonus_points(
                        pool,
                        seedtime_and_bonus_points_interval_secs,
                        formula_sql,
                    )
                },
            ))
        },
    )?;
    sched.add(seedtime_and_bonus_job).await?;

    let pool_user_torrent_stats = Arc::clone(&store.pool);
    let user_torrent_stats_job = Job::new_repeated_async(
        Duration::from_secs(store.config.user_torrent_stats_update_seconds),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_user_torrent_stats);
            Box::pin(instrument_periodic_task(
                instruments(),
                "user_torrent_stats",
                move || update_user_torrent_stats(pool),
            ))
        },
    )?;
    sched.add(user_torrent_stats_job).await?;

    let pool_inactive_users = Arc::clone(&store.pool);
    let inactive_user_ban_job = Job::new_repeated_async(
        Duration::from_secs(store.config.inactive_user_ban_seconds),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_inactive_users);
            Box::pin(instrument_periodic_task(
                instruments(),
                "inactive_user_ban",
                move || ban_inactive_users(pool),
            ))
        },
    )?;
    sched.add(inactive_user_ban_job).await?;

    let pool_expired_warnings = Arc::clone(&store.pool);
    let expired_warnings_job = Job::new_repeated_async(
        Duration::from_secs(store.config.expired_warnings_seconds),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_expired_warnings);
            Box::pin(instrument_periodic_task(
                instruments(),
                "expired_warnings",
                move || clear_expired_warnings(pool),
            ))
        },
    )?;
    sched.add(expired_warnings_job).await?;

    let pool_artist_peers = Arc::clone(&store.pool);
    let artist_peer_stats_job = Job::new_repeated_async(
        Duration::from_secs(store.config.artist_peer_stats_update_seconds),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_artist_peers);
            Box::pin(instrument_periodic_task(
                instruments(),
                "artist_peer_stats",
                move || update_artist_peer_stats(pool),
            ))
        },
    )?;
    sched.add(artist_peer_stats_job).await?;

    let pool_materialized_views = Arc::clone(&store.pool);
    let mut materialized_view_refresh_job = Job::new_repeated_async(
        Duration::from_secs(store.config.materialized_view_refresh_seconds),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_materialized_views);
            Box::pin(instrument_periodic_task(
                instruments(),
                "materialized_view_refresh",
                move || refresh_title_group_hierarchy_lite(pool),
            ))
        },
    )?;
    // first tick on startup instead of one interval after it
    let mut materialized_view_refresh_job_data = materialized_view_refresh_job.job_data()?;
    materialized_view_refresh_job_data.next_tick = chrono::Utc::now().timestamp() as u64;
    materialized_view_refresh_job.set_job_data(materialized_view_refresh_job_data)?;
    sched.add(materialized_view_refresh_job).await?;

    let pool_user_badges = Arc::clone(&store.pool);
    let user_badges_job = Job::new_repeated_async(
        Duration::from_secs(store.config.user_badges_evaluation_seconds),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_user_badges);
            Box::pin(instrument_periodic_task(
                instruments(),
                "user_badges_evaluation",
                move || evaluate_user_badges(pool),
            ))
        },
    )?;
    sched.add(user_badges_job).await?;

    let pool_announce_errors = Arc::clone(&store.pool);
    let announce_errors_retention_seconds = store.config.announce_errors_retention_seconds;
    let announce_errors_cleanup_job = Job::new_repeated_async(
        Duration::from_secs(store.config.announce_errors_cleanup_seconds),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_announce_errors);
            Box::pin(instrument_periodic_task(
                instruments(),
                "announce_errors_cleanup",
                move || {
                    remove_resolved_and_stale_announce_errors(
                        pool,
                        announce_errors_retention_seconds,
                    )
                },
            ))
        },
    )?;
    sched.add(announce_errors_cleanup_job).await?;

    let pool_reseed_requests = Arc::clone(&store.pool);
    let reseed_request_cleanup_job = Job::new_repeated_async(
        Duration::from_secs(store.config.reseed_request_cleanup_seconds),
        move |_uuid, _l| {
            let pool = Arc::clone(&pool_reseed_requests);
            Box::pin(instrument_periodic_task(
                instruments(),
                "reseed_request_cleanup",
                move || remove_resolved_reseed_requests(pool),
            ))
        },
    )?;
    sched.add(reseed_request_cleanup_job).await?;

    sched.start().await?;

    Ok(sched)
}
