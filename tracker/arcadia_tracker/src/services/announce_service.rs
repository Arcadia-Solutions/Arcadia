use std::collections::HashSet;

use arcadia_shared::{
    tracker::models::{env::SnatchedTorrentBonusPointsTransferredTo, peer_id::PeerId},
    utils::format_title_group_name,
};
use sqlx::PgPool;

use crate::announce::error::AnnounceError;

/// Check and deduct bonus points snatch cost for a new leech.
///
/// This function performs atomic queries that:
/// 1. Gets the torrent's bonus_points_snatch_cost and uploader ID
/// 2. Checks if the user already paid for this torrent:
///    - an in-progress leech: a torrent_activities row with downloaded > 0, or a peers row with
///      seeder = false (catches users who disconnected between leech attempts so their in-memory
///      db peer is gone), and no completed_at on their torrent_activities row
///    - or, when charge_on_resnatch is disabled, a previously completed snatch: the snatch
///      cost is then paid at most once per torrent and per user
///
///    A grab-only torrent_activities row (the .torrent file was downloaded but the user
///    never announced) never counts as already paid.
/// 3. Deducts points if: cost > 0, user is not uploader, has not paid already, has enough points.
///    The deducted amount is `bonus_points_snatch_cost * cost_factor / 100` (cost_factor is a
///    site-wide percentage, 100 = unchanged).
/// 4. Optionally transfers `bonus_points_snatch_cost * reward_factor / 100` to the uploader or the
///    current seeders (reward_factor is a site-wide percentage, 100 = unchanged). The cost and
///    reward factors are independent.
pub async fn check_and_deduct_snatch_cost(
    pool: &PgPool,
    torrent_id: u32,
    user_id: u32,
    transfer_to: Option<&SnatchedTorrentBonusPointsTransferredTo>,
    charge_on_resnatch: bool,
    cost_factor: i16,
    reward_factor: i16,
) -> Result<(), AnnounceError> {
    let mut tx = pool.begin().await.map_err(|e| {
        log::error!(
            "check_and_deduct_snatch_cost failed: could not begin transaction: {}",
            e
        );
        AnnounceError::InternalTrackerError
    })?;

    let row = sqlx::query!(
        r#"
        WITH torrent_info AS (
            SELECT bonus_points_snatch_cost, created_by_id, edition_group_id
            FROM torrents WHERE id = $1
        ),
        title_group_info AS (
            SELECT tg.name AS title_group_name, s.name AS series_name
            FROM torrent_info ti
            JOIN edition_groups eg ON eg.id = ti.edition_group_id
            JOIN title_groups tg ON tg.id = eg.title_group_id
            LEFT JOIN series s ON s.id = tg.series_id
        ),
        already_paid_activity AS (
            -- torrent_activities is UNIQUE (torrent_id, user_id), so all the
            -- torrent_activities checks below inspect the user's single row.
            -- already paid if the user has an in-progress leech (partial download
            -- was flushed to torrent_activities, or a persisted peers row: e.g. the
            -- user disconnected between 2 leech attempts so their in-memory peer is
            -- gone but the persisted peer row is still there) and never completed the
            -- torrent, or if they completed a snatch before and re-snatches are not
            -- charged. a grab-only row (the .torrent file was downloaded but the user
            -- never announced) never counts as already paid.
            SELECT 1 WHERE
              EXISTS (
                SELECT 1 FROM torrent_activities
                WHERE torrent_id = $1 AND user_id = $2
                  AND (
                    (downloaded > 0 AND completed_at IS NULL)
                    OR (completed_at IS NOT NULL AND $3 = false)
                  )
              )
              OR (
                EXISTS (
                    SELECT 1 FROM peers
                    WHERE torrent_id = $1 AND user_id = $2 AND seeder = false
                )
                AND NOT EXISTS (
                    SELECT 1 FROM torrent_activities
                    WHERE torrent_id = $1 AND user_id = $2 AND completed_at IS NOT NULL
                )
              )
        ),
        deduction AS (
            UPDATE users SET bonus_points = bonus_points - (SELECT bonus_points_snatch_cost FROM torrent_info) * $4 / 100
            WHERE id = $2
              AND (SELECT bonus_points_snatch_cost FROM torrent_info) > 0
              AND $2 != (SELECT created_by_id FROM torrent_info)
              AND bonus_points >= (SELECT bonus_points_snatch_cost FROM torrent_info) * $4 / 100
              -- we do this check in case the user only partially downloaded the torrent, sent a stopped event, and started leeching again
              -- the peer is removed from the in-memory db at a stopped event, and would be considered a new leecher
              AND NOT EXISTS (SELECT 1 FROM already_paid_activity)
            RETURNING id
        )
        SELECT
            (SELECT bonus_points_snatch_cost FROM torrent_info) AS cost,
            (SELECT bonus_points_snatch_cost FROM torrent_info) * $4 / 100 AS charged,
            (SELECT created_by_id FROM torrent_info) AS uploader_id,
            EXISTS (SELECT 1 FROM deduction) AS deducted,
            EXISTS (SELECT 1 FROM already_paid_activity) AS has_already_paid_activity,
            (SELECT username FROM users WHERE id = $2) AS "username!",
            (SELECT title_group_name FROM title_group_info) AS "title_group_name!",
            (SELECT series_name FROM title_group_info) AS "title_group_series_name?"
        "#,
        torrent_id as i32,
        user_id as i32,
        charge_on_resnatch,
        cost_factor as i32,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        log::error!("check_and_deduct_snatch_cost failed: could not check/deduct bonus points: {}", e);
        AnnounceError::InternalTrackerError
    })?;

    let cost = row.cost.unwrap_or(0);
    // Amount actually charged to the snatcher, after the site-wide cost factor.
    let charged = row.charged.unwrap_or(0);
    // Amount transferred to the uploader or current seeders, after the site-wide reward factor.
    let received = cost * reward_factor as i64 / 100;
    let is_uploader = row
        .uploader_id
        .map(|id| id as u32 == user_id)
        .unwrap_or(false);
    let deducted = row.deducted.unwrap_or(false);
    let has_already_paid_activity = row.has_already_paid_activity.unwrap_or(false);

    let username = row.username;
    let title_group_name = format_title_group_name(
        row.title_group_series_name.as_deref(),
        &row.title_group_name,
    );

    // If cost > 0, user is not uploader, has not paid already, and deduction failed
    if cost > 0 && !is_uploader && !has_already_paid_activity && !deducted {
        log::error!(
            "check_and_deduct_snatch_cost failed: user=\"{}\" (id={}) has insufficient bonus points for torrent_id={}, charged={}",
            username,
            user_id,
            torrent_id,
            charged
        );
        return Err(AnnounceError::InsufficientBonusPoints(charged));
    }

    // Transfer bonus points if deduction happened and transfer is configured
    if deducted {
        sqlx::query!(
            r#"
            INSERT INTO bonus_points_logs (user_id, action, amount, details, item_id)
            VALUES ($1, 'snatch_cost_deduction'::bonus_points_log_action_enum, $2, $3, $4)
            "#,
            user_id as i32,
            -charged,
            title_group_name,
            torrent_id as i64,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            log::error!(
                "check_and_deduct_snatch_cost failed: could not log snatch cost deduction: {}",
                e
            );
            AnnounceError::InternalTrackerError
        })?;

        match transfer_to {
            Some(SnatchedTorrentBonusPointsTransferredTo::Uploader) => {
                sqlx::query!(
                    r#"
                    WITH updated_uploader AS (
                        UPDATE users SET bonus_points = bonus_points + $1
                        WHERE id = (SELECT created_by_id FROM torrents WHERE id = $2)
                        RETURNING id
                    )
                    INSERT INTO bonus_points_logs (user_id, action, amount, details, item_id)
                    SELECT id, 'snatch_cost_received_as_uploader'::bonus_points_log_action_enum, $1, $3, $4
                    FROM updated_uploader
                    "#,
                    received,
                    torrent_id as i32,
                    title_group_name,
                    torrent_id as i64,
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    log::error!("check_and_deduct_snatch_cost failed: could not transfer bonus points to uploader: {}", e);
                    AnnounceError::InternalTrackerError
                })?;
            }
            Some(SnatchedTorrentBonusPointsTransferredTo::CurrentSeeders) => {
                sqlx::query!(
                    r#"
                    WITH seeder_info AS (
                        SELECT DISTINCT user_id, COUNT(*) OVER () as seeder_count
                        FROM peers
                        WHERE torrent_id = $1 AND seeder = true AND active = true
                    ),
                    per_seeder AS (
                        SELECT $2 / (SELECT seeder_count FROM seeder_info LIMIT 1) AS amount
                    ),
                    updated_seeders AS (
                        UPDATE users SET bonus_points = bonus_points + (SELECT amount FROM per_seeder)
                        WHERE id IN (SELECT user_id FROM seeder_info)
                        RETURNING id
                    )
                    INSERT INTO bonus_points_logs (user_id, action, amount, details, item_id)
                    SELECT id, 'snatch_cost_received_as_seeder'::bonus_points_log_action_enum, (SELECT amount FROM per_seeder), $3, $4
                    FROM updated_seeders
                    "#,
                    torrent_id as i32,
                    received,
                    title_group_name,
                    torrent_id as i64,
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    log::error!("check_and_deduct_snatch_cost failed: could not transfer bonus points to seeders: {}", e);
                    AnnounceError::InternalTrackerError
                })?;
            }
            None => {}
        }
    }

    tx.commit().await.map_err(|e| {
        log::error!(
            "check_and_deduct_snatch_cost failed: could not commit transaction: {}",
            e
        );
        AnnounceError::InternalTrackerError
    })?;

    match sqlx::query!(
        r#"
        SELECT
            tg.id AS title_group_id,
            tg.name AS title_group_name
        FROM torrents t
        LEFT JOIN edition_groups eg ON eg.id = t.edition_group_id
        LEFT JOIN title_groups tg ON tg.id = eg.title_group_id
        WHERE t.id = $1
        "#,
        torrent_id as i32,
    )
    .fetch_one(pool)
    .await
    {
        Ok(tg_row) => {
            log::info!(
                "check_and_deduct_snatch_cost succeeded: user=\"{}\" (id={}), title_group=\"{:?}\" (title_group_id={:?}, torrent_id={}), cost={}, is_uploader={}, has_already_paid_activity={}, deducted={}, transfer_to={:?}",
                username,
                user_id,
                tg_row.title_group_name,
                tg_row.title_group_id,
                torrent_id,
                cost,
                is_uploader,
                has_already_paid_activity,
                deducted,
                transfer_to
            );
        }
        Err(e) => {
            log::error!(
                "Failed to fetch log info for check_and_deduct_snatch_cost: {}",
                e
            );
        }
    }

    Ok(())
}

pub fn is_torrent_client_allowed(
    peer_id: &PeerId,
    allowed_torrent_clients: &HashSet<Vec<u8>>,
) -> bool {
    let peer_id_without_hyphen = &peer_id.0[1..];
    for prefix in allowed_torrent_clients.iter() {
        if peer_id_without_hyphen.starts_with(prefix) {
            return true;
        }
    }
    false
}
