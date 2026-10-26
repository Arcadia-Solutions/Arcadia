pub mod common;
pub mod mocks;

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::{artist::ArtistRole, entity::EntityRole};
use common::{auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::borrow::Borrow;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_merge_title_groups"),
    migrations = "../storage/migrations"
)]
async fn test_merge_title_groups_moves_all_related_data(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::MergeTitleGroup,
    )
    .await;

    let pg_pool: &PgPool = (*pool).borrow();

    // Verify initial state: source (id=10) has data
    let source_edition_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM edition_groups WHERE title_group_id = 10")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(source_edition_count.0, 1);

    let source_comment_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM title_group_comments WHERE title_group_id = 10")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(source_comment_count.0, 2);

    // Perform merge: source=10 into target=11
    let req = test::TestRequest::post()
        .uri("/api/title-groups/merge?source_title_group_id=10&target_title_group_id=11")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Source title group should be deleted
    let source_result = pool.find_title_group(10).await;
    assert!(source_result.is_err());

    // Target title group should still exist
    let target = pool.find_title_group(11).await;
    assert!(target.is_ok());

    // Edition groups: source's edition should now be on target (1 source + 1 target = 2)
    let target_edition_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM edition_groups WHERE title_group_id = 11")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(target_edition_count.0, 2);

    // Affiliated artists: Artist A (source-only) moved, Artist Shared (conflict) kept target's, Artist B (target-only) stays
    // Result: artists 10, 11, 12 should all be affiliated with target
    let target_artist_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM affiliated_artists WHERE title_group_id = 11")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(target_artist_count.0, 3);

    // Tags: tag_a (source-only) moved, tag_shared (conflict) kept target's, tag_b (target-only) stays
    let target_tag_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM title_group_applied_tags WHERE title_group_id = 11")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(target_tag_count.0, 3);

    // Comments: all 3 (2 from source + 1 already on target)
    let target_comment_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM title_group_comments WHERE title_group_id = 11")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(target_comment_count.0, 3);

    // Bookmarks: user 100 (source-only) moved, user 146 (conflict) kept target's
    let target_bookmark_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM title_group_bookmarks WHERE title_group_id = 11")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(target_bookmark_count.0, 2);

    // Subscriptions (title_group_torrents): user 100 (source-only) moved, user 146 (conflict) kept target's
    let target_sub_torrent_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM subscriptions_title_group_torrents WHERE title_group_id = 11",
    )
    .fetch_one(pg_pool)
    .await
    .unwrap();
    assert_eq!(target_sub_torrent_count.0, 2);

    // Subscriptions (title_group_comments): user 100 (source-only) moved, user 146 (conflict) kept target's
    let target_sub_comment_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM subscriptions_title_group_comments WHERE title_group_id = 11",
    )
    .fetch_one(pg_pool)
    .await
    .unwrap();
    assert_eq!(target_sub_comment_count.0, 2);

    // Collage entries: collage 10 had both (conflict, source deleted), collage 11 had source only (moved)
    let target_collage_entry_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM collage_entry WHERE title_group_id = 11")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(target_collage_entry_count.0, 2);

    // Torrent request: moved from source to target
    let target_request_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM torrent_requests WHERE title_group_id = 11")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(target_request_count.0, 1);

    // External links: source had {source-only, shared}, target had {target-only, shared}
    // After merge: target should have all 3 unique links
    let target_title_group = pool.find_title_group(11).await.unwrap();
    assert_eq!(target_title_group.external_links.len(), 3);
    assert!(target_title_group
        .external_links
        .contains(&"https://source-only.example.com".to_string()));
    assert!(target_title_group
        .external_links
        .contains(&"https://target-only.example.com".to_string()));
    assert!(target_title_group
        .external_links
        .contains(&"https://shared.example.com".to_string()));

    // Conflicts keep the target's row
    let target_hierarchy = pool.find_title_group_hierarchy(11, 146).await.unwrap();
    let shared_artist = target_hierarchy
        .affiliated_artists
        .iter()
        .find(|affiliated_artist| affiliated_artist.artist_id == 12)
        .unwrap();
    assert_eq!(shared_artist.roles, vec![ArtistRole::Guest]);
    assert_eq!(target_hierarchy.affiliated_entities.len(), 2);
    let shared_entity = target_hierarchy
        .affiliated_entities
        .iter()
        .find(|affiliated_entity| affiliated_entity.entity_id == 11)
        .unwrap();
    assert!(matches!(shared_entity.roles[..], [EntityRole::Label]));
    assert_eq!(target_hierarchy.bookmark_id, Some(12));
    assert!(target_hierarchy.is_subscribed_to_torrents);
    assert!(target_hierarchy.is_subscribed_to_comments);

    // Notifications follow the merged title group
    let notifications = pool.find_all_notifications(100, true).await.unwrap();
    assert_eq!(notifications.title_group_comments.len(), 1);
    assert_eq!(notifications.title_group_comments[0].title_group_id, 11);
    assert_eq!(notifications.artist_title_groups.len(), 1);
    assert_eq!(notifications.artist_title_groups[0].title_group_id, 11);

    // No data should remain referencing source title group
    let source_anything: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM edition_groups WHERE title_group_id = 10")
            .fetch_one(pg_pool)
            .await
            .unwrap();
    assert_eq!(source_anything.0, 0);
}

#[sqlx::test(
    fixtures("with_test_users", "with_merge_title_groups"),
    migrations = "../storage/migrations"
)]
async fn test_cannot_merge_title_groups_with_different_content_types(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::MergeTitleGroup,
    )
    .await;

    // Title group 10 is 'music', title group 12 is 'software'
    let req = test::TestRequest::post()
        .uri("/api/title-groups/merge?source_title_group_id=10&target_title_group_id=12")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Both title groups should still exist
    assert!(pool.find_title_group(10).await.is_ok());
    assert!(pool.find_title_group(12).await.is_ok());
}
