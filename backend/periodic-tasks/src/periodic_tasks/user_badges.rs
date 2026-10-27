use arcadia_common::error::Result;
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::user_badge::{UserBadge, UserBadgeCriteria, UserBadgeType},
};
use std::collections::HashSet;
use std::sync::Arc;

pub async fn evaluate_user_badges(pool: Arc<ConnectionPool>) -> Result<u64> {
    let (awarded, revoked) = evaluate_user_badges_inner(pool).await?;
    log::info!(
        "User badges evaluation: {} awarded, {} revoked",
        awarded,
        revoked
    );
    Ok((awarded + revoked) as u64)
}

pub async fn evaluate_user_badges_inner(pool: Arc<ConnectionPool>) -> Result<(usize, usize)> {
    let badges = pool.find_all_user_badges_full().await?;
    let auto_badges: Vec<UserBadge> = badges
        .into_iter()
        .filter(|b| !matches!(b.badge_type, UserBadgeType::Manual))
        .collect();

    if auto_badges.is_empty() {
        return Ok((0, 0));
    }

    let mut total_awarded: usize = 0;
    let mut total_revoked: usize = 0;

    for badge in &auto_badges {
        let criteria_value = match badge.criteria.as_ref() {
            Some(value) => value,
            None => {
                log::warn!("Auto badge {} has no criteria, skipping", badge.id);
                continue;
            }
        };

        let criteria: UserBadgeCriteria = match serde_json::from_value(criteria_value.clone()) {
            Ok(criteria) => criteria,
            Err(e) => {
                log::warn!(
                    "Could not deserialize criteria for badge {}: {}",
                    badge.id,
                    e
                );
                continue;
            }
        };

        let qualifying_ids: HashSet<i32> = match qualifying_user_ids(&pool, &criteria).await {
            Ok(ids) => ids.into_iter().collect(),
            Err(e) => {
                log::warn!("Could not evaluate badge {}: {}", badge.id, e);
                continue;
            }
        };

        let existing = pool.find_user_ids_with_badge(badge.id).await?;
        let already_awarded: HashSet<i32> = existing.iter().map(|(uid, _)| *uid).collect();
        let auto_awarded: HashSet<i32> = existing
            .iter()
            .filter(|(_, awarded_by)| awarded_by.is_none())
            .map(|(uid, _)| *uid)
            .collect();

        for &user_id in qualifying_ids.difference(&already_awarded) {
            if let Err(e) = pool.award_user_badge(user_id, badge.id, None, None).await {
                log::warn!(
                    "Could not award badge {} to user {}: {}",
                    badge.id,
                    user_id,
                    e
                );
                continue;
            }
            total_awarded += 1;
        }

        if badge.revoke_when_criteria_unmet {
            for &user_id in auto_awarded.difference(&qualifying_ids) {
                match pool.revoke_auto_earned_badge(user_id, badge.id).await {
                    Ok(true) => total_revoked += 1,
                    Ok(false) => {}
                    Err(e) => {
                        log::warn!(
                            "Could not revoke badge {} from user {}: {}",
                            badge.id,
                            user_id,
                            e
                        );
                    }
                }
            }
        }
    }

    Ok((total_awarded, total_revoked))
}

async fn qualifying_user_ids(
    pool: &Arc<ConnectionPool>,
    criteria: &UserBadgeCriteria,
) -> Result<Vec<i32>> {
    match criteria {
        UserBadgeCriteria::TorrentsUploaded {
            search,
            minimum_title_group_amount,
        } => {
            pool.find_qualifying_uploader_ids(search, *minimum_title_group_amount)
                .await
        }
        UserBadgeCriteria::ForumPosts {
            minimum_post_character_count,
            required_substring,
            minimum_post_amount,
        } => {
            pool.find_qualifying_forum_post_user_ids(
                *minimum_post_character_count,
                required_substring.as_deref(),
                *minimum_post_amount,
            )
            .await
        }
        UserBadgeCriteria::ForumThreads {
            minimum_thread_name_character_count,
            required_substring,
            minimum_thread_amount,
        } => {
            pool.find_qualifying_forum_thread_user_ids(
                *minimum_thread_name_character_count,
                required_substring.as_deref(),
                *minimum_thread_amount,
            )
            .await
        }
        UserBadgeCriteria::AdventCalendarDoors {
            calendar_id,
            minimum_opened_door_amount,
            door_names,
        } => {
            pool.find_qualifying_advent_calendar_door_user_ids(
                *calendar_id,
                *minimum_opened_door_amount,
                door_names,
            )
            .await
        }
    }
}
