use std::num::NonZeroU64;

use serde::Deserialize;

/// The `periodic_tasks` section of the `config.yml` file. The service running the tasks owns the
/// rest of the configuration, and hands this section over.
#[derive(Clone, Debug, Deserialize)]
pub struct PeriodicTasksConfig {
    /// Custom formula for bonus points calculation.
    /// Available variables: `seedtime`, `seeders` and `size`.
    pub bonus_points_formula: String,
    /// Interval for user class promotion/demotion checks, in seconds.
    pub user_class_changes_seconds: u64,
    /// Interval for seedtime and bonus points updates, in seconds.
    pub seedtime_and_bonus_points_update_seconds: u64,
    /// Interval for user torrent stats updates (seeding, leeching, snatched, seeding size),
    /// in seconds.
    pub user_torrent_stats_update_seconds: u64,
    /// Interval for checking and banning inactive users, in seconds.
    pub inactive_user_ban_seconds: u64,
    /// Interval for refreshing the `title_group_hierarchy_lite` materialized view, in seconds.
    pub materialized_view_refresh_seconds: u64,
    /// Interval for clearing expired user warnings, in seconds.
    pub expired_warnings_seconds: u64,
    /// Interval for aggregating artist peer stats from torrents, in seconds.
    pub artist_peer_stats_update_seconds: u64,
    /// Interval for evaluating user auto-badges, in seconds.
    pub user_badges_evaluation_seconds: u64,
    /// Interval for removing the resolved and stale announce errors, in seconds.
    pub announce_errors_cleanup_seconds: u64,
    /// Announce errors not seen again within this delay are considered stale and removed,
    /// in seconds. Must be longer than the tracker's announce interval.
    pub announce_errors_retention_seconds: u64,
    /// Interval for asking the providers (TMDB and the scraper plugins) for the ratings due, in
    /// seconds.
    pub public_ratings_fetch_seconds: u64,
    /// A rating is asked for again this long after the last attempt, whatever its outcome, in
    /// seconds. The scraper plugins decide themselves whether their rating needs a refresh.
    /// Non zero, or a rating would be due again as soon as it is asked for, and fetched endlessly.
    pub public_ratings_retry_seconds: NonZeroU64,
    /// Maximum amount of ratings asked for at each fetch.
    pub public_ratings_batch_size: i64,
    /// Interval for removing the reseed requests of torrents that are healthy again, in seconds.
    #[serde(default = "default_reseed_request_cleanup_seconds")]
    pub reseed_request_cleanup_seconds: u64,
}

fn default_reseed_request_cleanup_seconds() -> u64 {
    900
}

/// Validates and converts a formula string to SQL expression.
/// Replaces variable names with their SQL equivalents:
/// - seedtime: total seed time in seconds
/// - seeders: replaced by the given `seeders_expression`
/// - size: torrent size in bytes
pub fn formula_to_sql(formula: &str, seeders_expression: &str) -> Result<String, &'static str> {
    for c in formula.chars() {
        if !c.is_alphanumeric()
            && c != ' '
            && c != '+'
            && c != '-'
            && c != '*'
            && c != '/'
            && c != '('
            && c != ')'
            && c != '.'
            && c != '_'
        {
            return Err("Formula contains invalid characters");
        }
    }

    let sql = formula
        .replace("seedtime", "ta.total_seed_time")
        .replace("seeders", seeders_expression)
        .replace("size", "t.size");

    Ok(sql)
}
