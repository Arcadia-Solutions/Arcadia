pub mod common;
pub mod mocks;

use arcadia_periodic_tasks::periodic_tasks::user_badges::evaluate_user_badges_inner;
use arcadia_storage::connection_pool::ConnectionPool;
use sqlx::PgPool;
use std::sync::Arc;

const FORUM_POSTER_BADGE_ID: i32 = 700;
const FORUM_CENTURION_BADGE_ID: i32 = 701;
const NO_REVOKE_BADGE_ID: i32 = 702;
const UPLOADER_TWO_BADGE_ID: i32 = 800;
const LOVE_SINGLES_TWO_BADGE_ID: i32 = 801;
const LOVE_SINGLES_ONE_BADGE_ID: i32 = 802;
const ADVENT_CALENDAR_BADGE_ID: i32 = 900;
const ADVENT_CALENDAR_DAY_2_BADGE_ID: i32 = 901;
const QUALIFYING_USER_ID: i32 = 100;
const SEED_CREATOR_USER_ID: i32 = 1;

async fn user_has_badge(pool: &PgPool, user_id: i32, badge_id: i32) -> bool {
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM user_earned_badges WHERE user_id = $1 AND badge_id = $2)",
    )
    .bind(user_id)
    .bind(badge_id)
    .fetch_one(pool)
    .await
    .unwrap();
    exists.unwrap_or(false)
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_user_badges_evaluation"
    ),
    migrations = "../storage/migrations"
)]
async fn test_awards_badge_to_qualifying_user(pool: PgPool) {
    let raw_pool = pool.clone();
    let conn_pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (awarded, revoked) = evaluate_user_badges_inner(conn_pool).await.unwrap();
    assert_eq!(awarded, 1);
    assert_eq!(revoked, 0);

    assert!(user_has_badge(&raw_pool, QUALIFYING_USER_ID, FORUM_POSTER_BADGE_ID).await);
    assert!(!user_has_badge(&raw_pool, QUALIFYING_USER_ID, FORUM_CENTURION_BADGE_ID).await);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_user_badges_evaluation"
    ),
    migrations = "../storage/migrations"
)]
async fn test_evaluation_is_idempotent(pool: PgPool) {
    let raw_pool = pool.clone();
    let conn_pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let (awarded_first, _) = evaluate_user_badges_inner(Arc::clone(&conn_pool))
        .await
        .unwrap();
    assert_eq!(awarded_first, 1);

    let (awarded_second, revoked_second) = evaluate_user_badges_inner(conn_pool).await.unwrap();
    assert_eq!(awarded_second, 0);
    assert_eq!(revoked_second, 0);

    let earned_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) as "count!" FROM user_earned_badges WHERE user_id = $1 AND badge_id = $2"#,
    )
    .bind(QUALIFYING_USER_ID)
    .bind(FORUM_POSTER_BADGE_ID)
    .fetch_one(&raw_pool)
    .await
    .unwrap();
    assert_eq!(earned_count, 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_user_badges_evaluation"
    ),
    migrations = "../storage/migrations"
)]
async fn test_revokes_when_criteria_unmet_for_auto_award(pool: PgPool) {
    let raw_pool = pool.clone();

    // Pre-seed an auto award (awarded_by_id IS NULL) for a badge user 100 does NOT qualify for
    sqlx::query(
        r#"INSERT INTO user_earned_badges (user_id, badge_id, awarded_by_id) VALUES ($1, $2, NULL)"#,
    )
    .bind(QUALIFYING_USER_ID)
    .bind(FORUM_CENTURION_BADGE_ID)
    .execute(&raw_pool)
    .await
    .unwrap();

    let conn_pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (_, revoked) = evaluate_user_badges_inner(conn_pool).await.unwrap();
    assert_eq!(revoked, 1);

    assert!(!user_has_badge(&raw_pool, QUALIFYING_USER_ID, FORUM_CENTURION_BADGE_ID).await);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_user_badges_evaluation"
    ),
    migrations = "../storage/migrations"
)]
async fn test_does_not_revoke_when_revoke_flag_off(pool: PgPool) {
    let raw_pool = pool.clone();

    // Pre-seed auto award for badge 702 (revoke_when_criteria_unmet=FALSE), criteria is unmet
    sqlx::query(
        r#"INSERT INTO user_earned_badges (user_id, badge_id, awarded_by_id) VALUES ($1, $2, NULL)"#,
    )
    .bind(QUALIFYING_USER_ID)
    .bind(NO_REVOKE_BADGE_ID)
    .execute(&raw_pool)
    .await
    .unwrap();

    let conn_pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (_, revoked) = evaluate_user_badges_inner(conn_pool).await.unwrap();
    assert_eq!(revoked, 0);

    assert!(user_has_badge(&raw_pool, QUALIFYING_USER_ID, NO_REVOKE_BADGE_ID).await);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_refreshed_title_group_hierarchy_lite",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_user_badges_evaluation"
    ),
    migrations = "../storage/migrations"
)]
async fn test_awards_torrents_uploaded_badge_with_search_filters(pool: PgPool) {
    let raw_pool = pool.clone();
    let conn_pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    evaluate_user_badges_inner(conn_pool).await.unwrap();

    // User 1 ("creator") owns 2 title groups (Love Me Do single, RollerCoaster Tycoon game).
    // Badge 800: unfiltered, threshold 2 → user 1 qualifies.
    assert!(user_has_badge(&raw_pool, SEED_CREATOR_USER_ID, UPLOADER_TWO_BADGE_ID).await);

    // Badge 801: filters (name~"Love", content_type=music, category=Single) match only TG 1.
    // Threshold 2, but filtered count is 1 → user 1 does NOT qualify, proving the filter narrows.
    assert!(!user_has_badge(&raw_pool, SEED_CREATOR_USER_ID, LOVE_SINGLES_TWO_BADGE_ID).await);

    // Badge 802: same filter as 801 but threshold 1 → user 1 qualifies.
    assert!(user_has_badge(&raw_pool, SEED_CREATOR_USER_ID, LOVE_SINGLES_ONE_BADGE_ID).await);

    // User 100 has no torrents, qualifies for none of the torrent badges.
    assert!(!user_has_badge(&raw_pool, QUALIFYING_USER_ID, UPLOADER_TWO_BADGE_ID).await);
    assert!(!user_has_badge(&raw_pool, QUALIFYING_USER_ID, LOVE_SINGLES_ONE_BADGE_ID).await);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_refreshed_title_group_hierarchy_lite",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_user_badges_evaluation"
    ),
    migrations = "../storage/migrations"
)]
async fn test_revokes_torrents_uploaded_auto_award_when_threshold_unmet(pool: PgPool) {
    let raw_pool = pool.clone();

    // Pre-seed an auto award for badge 801 (filtered, threshold 2). User 1's filtered count is 1,
    // so the criteria is unmet and the badge should be revoked on evaluation.
    sqlx::query(
        r#"INSERT INTO user_earned_badges (user_id, badge_id, awarded_by_id) VALUES ($1, $2, NULL)"#,
    )
    .bind(SEED_CREATOR_USER_ID)
    .bind(LOVE_SINGLES_TWO_BADGE_ID)
    .execute(&raw_pool)
    .await
    .unwrap();

    let conn_pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    evaluate_user_badges_inner(conn_pool).await.unwrap();

    assert!(!user_has_badge(&raw_pool, SEED_CREATOR_USER_ID, LOVE_SINGLES_TWO_BADGE_ID).await);
    assert!(user_has_badge(&raw_pool, SEED_CREATOR_USER_ID, UPLOADER_TWO_BADGE_ID).await);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_advent_calendar_badge"),
    migrations = "../storage/migrations"
)]
async fn test_awards_advent_calendar_doors_badge(pool: PgPool) {
    let raw_pool = pool.clone();
    let conn_pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    evaluate_user_badges_inner(conn_pool).await.unwrap();

    // user 100 opened both doors, user 101 only one
    assert!(user_has_badge(&raw_pool, QUALIFYING_USER_ID, ADVENT_CALENDAR_BADGE_ID).await);
    assert!(!user_has_badge(&raw_pool, 101, ADVENT_CALENDAR_BADGE_ID).await);
    // the door names filter only counts Day 2, which user 101 did not open
    assert!(
        user_has_badge(
            &raw_pool,
            QUALIFYING_USER_ID,
            ADVENT_CALENDAR_DAY_2_BADGE_ID
        )
        .await
    );
    assert!(!user_has_badge(&raw_pool, 101, ADVENT_CALENDAR_DAY_2_BADGE_ID).await);
}
