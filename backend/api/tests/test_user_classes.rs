use crate::{common, mocks};

use actix_web::{http::StatusCode, test};
use arcadia_storage::{
    connection_pool::ConnectionPool,
    models::user::{
        DeleteUserClass, EditedUserClass, UpdatedUserPermissions, UserClass, UserClassChange,
        UserClassLockStatus, UserCreatedUserClass, UserPermission,
    },
};
use common::{
    TestUser, auth_header, call_and_read_body_json_with_status, create_test_app,
    create_test_app_and_login,
};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

// ============================================================================
// CREATE USER CLASS TESTS
// ============================================================================

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_can_create_user_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateUserClass).await;

    let user_class = UserCreatedUserClass {
        name: "power_user".into(),
        new_permissions: vec![
            UserPermission::UploadTorrent,
            UserPermission::DownloadTorrent,
        ],
        max_snatches_per_day: None,
        automatic_promotion: true,
        automatic_demotion: true,
        promotion_allowed_while_warned: false,
        previous_user_class: None,
        required_account_age_in_days: 0,
        required_ratio: 0.0,
        required_torrent_uploads: 0,
        required_torrent_uploads_in_unique_title_groups: 0,
        required_uploaded: 0,
        required_torrent_snatched: 0,
        required_downloaded: 0,
        required_forum_posts: 0,
        required_forum_posts_in_unique_threads: 0,
        required_title_group_comments: 0,
        required_seeding_size: 0,
        promotion_cost_bonus_points: 0,
    };

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-classes")
        .set_json(&user_class)
        .to_request();

    let created: UserClass =
        call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    assert_eq!(created.name, "power_user");
    assert_eq!(created.new_permissions.len(), 2);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_regular_user_cannot_create_user_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let user_class = UserCreatedUserClass {
        name: "power_user".into(),
        new_permissions: vec![],
        max_snatches_per_day: None,
        automatic_promotion: true,
        automatic_demotion: true,
        promotion_allowed_while_warned: false,
        previous_user_class: None,
        required_account_age_in_days: 0,
        required_ratio: 0.0,
        required_torrent_uploads: 0,
        required_torrent_uploads_in_unique_title_groups: 0,
        required_uploaded: 0,
        required_torrent_snatched: 0,
        required_downloaded: 0,
        required_forum_posts: 0,
        required_forum_posts_in_unique_threads: 0,
        required_title_group_comments: 0,
        required_seeding_size: 0,
        promotion_cost_bonus_points: 0,
    };

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-classes")
        .set_json(&user_class)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_create_user_class_requires_auth(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let service = create_test_app(pool, MockRedisPool::default()).await;

    let user_class = UserCreatedUserClass {
        name: "power_user".into(),
        new_permissions: vec![],
        max_snatches_per_day: None,
        automatic_promotion: true,
        automatic_demotion: true,
        promotion_allowed_while_warned: false,
        previous_user_class: None,
        required_account_age_in_days: 0,
        required_ratio: 0.0,
        required_torrent_uploads: 0,
        required_torrent_uploads_in_unique_title_groups: 0,
        required_uploaded: 0,
        required_torrent_snatched: 0,
        required_downloaded: 0,
        required_forum_posts: 0,
        required_forum_posts_in_unique_threads: 0,
        required_title_group_comments: 0,
        required_seeding_size: 0,
        promotion_cost_bonus_points: 0,
    };

    let req = test::TestRequest::post()
        .uri("/api/user-classes")
        .set_json(&user_class)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_create_user_class_with_invalid_name(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::CreateUserClass).await;

    // Too short name
    let user_class = UserCreatedUserClass {
        name: "ab".into(),
        new_permissions: vec![],
        max_snatches_per_day: None,
        automatic_promotion: true,
        automatic_demotion: true,
        promotion_allowed_while_warned: false,
        previous_user_class: None,
        required_account_age_in_days: 0,
        required_ratio: 0.0,
        required_torrent_uploads: 0,
        required_torrent_uploads_in_unique_title_groups: 0,
        required_uploaded: 0,
        required_torrent_snatched: 0,
        required_downloaded: 0,
        required_forum_posts: 0,
        required_forum_posts_in_unique_threads: 0,
        required_title_group_comments: 0,
        required_seeding_size: 0,
        promotion_cost_bonus_points: 0,
    };

    let req = test::TestRequest::post()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-classes")
        .set_json(&user_class)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// EDIT USER CLASS TESTS
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_edit_user_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditUserClass).await;

    let edited = EditedUserClass {
        name: "advanced_user".into(),
        new_permissions: vec![UserPermission::EditArtist],
        max_snatches_per_day: None,
        automatic_promotion: true,
        automatic_demotion: true,
        promotion_allowed_while_warned: false,
        previous_user_class: None,
        required_account_age_in_days: 0,
        required_ratio: 0.0,
        required_torrent_uploads: 0,
        required_torrent_uploads_in_unique_title_groups: 0,
        required_uploaded: 0,
        required_torrent_snatched: 0,
        required_downloaded: 0,
        required_forum_posts: 0,
        required_forum_posts_in_unique_threads: 0,
        required_title_group_comments: 0,
        required_seeding_size: 0,
        promotion_cost_bonus_points: 0,
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-classes/test_class")
        .set_json(&edited)
        .to_request();

    let updated: UserClass =
        call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(updated.name, "advanced_user");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_edit_nonexistent_user_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditUserClass).await;

    let edited = EditedUserClass {
        name: "new_name".into(),
        new_permissions: vec![],
        max_snatches_per_day: None,
        automatic_promotion: true,
        automatic_demotion: true,
        promotion_allowed_while_warned: false,
        previous_user_class: None,
        required_account_age_in_days: 0,
        required_ratio: 0.0,
        required_torrent_uploads: 0,
        required_torrent_uploads_in_unique_title_groups: 0,
        required_uploaded: 0,
        required_torrent_snatched: 0,
        required_downloaded: 0,
        required_forum_posts: 0,
        required_forum_posts_in_unique_threads: 0,
        required_title_group_comments: 0,
        required_seeding_size: 0,
        promotion_cost_bonus_points: 0,
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-classes/nonexistent")
        .set_json(&edited)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// DELETE USER CLASS TESTS
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_can_delete_user_class_with_migration(pool: PgPool) {
    let pool_arc = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        Arc::clone(&pool_arc),
        MockRedisPool::default(),
        TestUser::DeleteUserClass,
    )
    .await;

    // Delete test_class and migrate users to empty_class
    let delete_body = DeleteUserClass {
        target_class_name: "empty_class".into(),
    };

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-classes/test_class")
        .set_json(&delete_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_delete_with_nonexistent_target_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::DeleteUserClass).await;

    let delete_body = DeleteUserClass {
        target_class_name: "nonexistent".into(),
    };

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-classes/test_class")
        .set_json(&delete_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_delete_nonexistent_user_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::DeleteUserClass).await;

    let delete_body = DeleteUserClass {
        target_class_name: "newbie".into(),
    };

    let req = test::TestRequest::delete()
        .insert_header(auth_header(&user.token))
        .uri("/api/user-classes/nonexistent")
        .set_json(&delete_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// EDIT USER PERMISSIONS TESTS
// ============================================================================

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_can_edit_user_permissions(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SetAndViewUserPermissions,
    )
    .await;

    let permissions = UpdatedUserPermissions {
        permissions: vec![UserPermission::EditArtist, UserPermission::EditSeries],
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/100/permissions")
        .set_json(&permissions)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(
    fixtures("with_test_users", "with_locked_user"),
    migrations = "../storage/migrations"
)]
async fn test_cannot_edit_permissions_of_locked_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::SetAndViewUserPermissions,
    )
    .await;

    let permissions = UpdatedUserPermissions {
        permissions: vec![UserPermission::EditArtist],
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/999/permissions")
        .set_json(&permissions)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ============================================================================
// LOCK USER CLASS TESTS
// ============================================================================

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_can_lock_user_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::LockUserClass).await;

    let lock_status = UserClassLockStatus { class_locked: true };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/100/lock-class")
        .set_json(&lock_status)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(fixtures("with_test_users"), migrations = "../storage/migrations")]
async fn test_staff_can_unlock_user_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::LockUserClass).await;

    let lock_status = UserClassLockStatus {
        class_locked: false,
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/100/lock-class")
        .set_json(&lock_status)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// ============================================================================
// CHANGE USER CLASS TESTS
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_change_user_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ChangeUserClass).await;

    let class_change = UserClassChange {
        class_name: "test_class".into(),
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/100/class")
        .set_json(&class_change)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(
    fixtures("with_test_users", "with_locked_user"),
    migrations = "../storage/migrations"
)]
async fn test_cannot_change_class_of_locked_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ChangeUserClass).await;

    let class_change = UserClassChange {
        class_name: "newbie".into(),
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/999/class")
        .set_json(&class_change)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_cannot_change_to_nonexistent_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ChangeUserClass).await;

    let class_change = UserClassChange {
        class_name: "nonexistent_class".into(),
    };

    let req = test::TestRequest::put()
        .insert_header(auth_header(&user.token))
        .uri("/api/users/100/class")
        .set_json(&class_change)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// 1-HOP PROMOTION/DEMOTION TESTS
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_hierarchy_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_one_hop_promotion(pool: PgPool) {
    let pool_arc = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Promote user from basic_class to advanced_class (1-hop promotion)
    // User 1000 is created in fixture with basic_class and download_torrent permission
    pool_arc
        .change_user_class(1000, "advanced_class", false)
        .await
        .expect("Failed to change user class");

    // Verify user now has permissions from both basic_class and advanced_class
    let user = pool_arc
        .find_user_with_id(1000)
        .await
        .expect("Failed to fetch user");

    assert_eq!(user.class_name, "advanced_class");
    assert!(user.permissions.contains(&UserPermission::DownloadTorrent)); // from basic_class
    assert!(user.permissions.contains(&UserPermission::UploadTorrent)); // from advanced_class
    assert_eq!(user.permissions.len(), 2);
}

#[sqlx::test(
    fixtures("with_test_users", "with_hierarchy_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_one_hop_demotion(pool: PgPool) {
    let pool_arc = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Demote user from advanced_class to basic_class (1-hop demotion)
    // User 1001 is created in fixture with advanced_class and both permissions
    pool_arc
        .change_user_class(1001, "basic_class", false)
        .await
        .expect("Failed to change user class");

    // Verify user now only has permissions from basic_class
    let user = pool_arc
        .find_user_with_id(1001)
        .await
        .expect("Failed to fetch user");

    assert_eq!(user.class_name, "basic_class");
    assert!(user.permissions.contains(&UserPermission::DownloadTorrent)); // from basic_class
    assert!(!user.permissions.contains(&UserPermission::UploadTorrent)); // removed from advanced_class
    assert_eq!(user.permissions.len(), 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_hierarchy_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_lateral_class_change_keeps_permissions(pool: PgPool) {
    let pool_arc = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Change to a class that is not in the hierarchy (lateral move to newbie)
    // User 1002 is created in fixture with basic_class and download_torrent permission
    pool_arc
        .change_user_class(1002, "newbie", false)
        .await
        .expect("Failed to change user class");

    // Verify permissions are unchanged (lateral move, not promotion/demotion)
    let user = pool_arc
        .find_user_with_id(1002)
        .await
        .expect("Failed to fetch user");

    assert_eq!(user.class_name, "newbie");
    // Permissions should remain unchanged since this is a lateral move
    assert!(user.permissions.contains(&UserPermission::DownloadTorrent));
    assert_eq!(user.permissions.len(), 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_hierarchy_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_edit_class_permissions_propagate_to_classes_above(pool: PgPool) {
    let pool_arc = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Edit basic_class: add EditArtist, drop DownloadTorrent.
    // Hierarchy: basic_class -> advanced_class -> elite_class, so users in
    // both advanced_class (1 level above) and elite_class (2 levels above)
    // must gain EditArtist and lose DownloadTorrent.
    let edited = EditedUserClass {
        name: "basic_class".into(),
        new_permissions: vec![UserPermission::EditArtist],
        max_snatches_per_day: None,
        automatic_promotion: true,
        automatic_demotion: true,
        promotion_allowed_while_warned: false,
        previous_user_class: None,
        required_account_age_in_days: 0,
        required_ratio: 0.0,
        required_torrent_uploads: 0,
        required_torrent_uploads_in_unique_title_groups: 0,
        required_uploaded: 0,
        required_torrent_snatched: 0,
        required_downloaded: 0,
        required_forum_posts: 0,
        required_forum_posts_in_unique_threads: 0,
        required_title_group_comments: 0,
        required_seeding_size: 0,
        promotion_cost_bonus_points: 0,
    };

    pool_arc
        .update_user_class("basic_class", &edited)
        .await
        .expect("Failed to update user class");

    // Direct class user (basic_class)
    let basic_user = pool_arc
        .find_user_with_id(1000)
        .await
        .expect("Failed to fetch basic_class user");
    assert!(basic_user.permissions.contains(&UserPermission::EditArtist));
    assert!(
        !basic_user
            .permissions
            .contains(&UserPermission::DownloadTorrent)
    );

    // Class above (advanced_class) - propagation target
    let advanced_user = pool_arc
        .find_user_with_id(1001)
        .await
        .expect("Failed to fetch advanced_class user");
    assert!(
        advanced_user
            .permissions
            .contains(&UserPermission::EditArtist)
    );
    assert!(
        !advanced_user
            .permissions
            .contains(&UserPermission::DownloadTorrent)
    );
    // advanced_class's own permission stays intact
    assert!(
        advanced_user
            .permissions
            .contains(&UserPermission::UploadTorrent)
    );

    // Class 2 levels above (elite_class) - transitive propagation
    let elite_user = pool_arc
        .find_user_with_id(1004)
        .await
        .expect("Failed to fetch elite_class user");
    assert!(elite_user.permissions.contains(&UserPermission::EditArtist));
    assert!(
        !elite_user
            .permissions
            .contains(&UserPermission::DownloadTorrent)
    );
    // elite_class's and advanced_class's own permissions stay intact
    assert!(
        elite_user
            .permissions
            .contains(&UserPermission::UploadTorrent)
    );
    assert!(
        elite_user
            .permissions
            .contains(&UserPermission::EditWikiArticle)
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_hierarchy_user_classes"),
    migrations = "../storage/migrations"
)]
async fn test_deduplicates_permissions(pool: PgPool) {
    let pool_arc = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Change to advanced_class (should deduplicate during the update)
    // User 1003 is created in fixture with basic_class and duplicate permissions
    pool_arc
        .change_user_class(1003, "advanced_class", false)
        .await
        .expect("Failed to change user class");

    // Verify permissions are deduplicated
    let user = pool_arc
        .find_user_with_id(1003)
        .await
        .expect("Failed to fetch user");

    assert_eq!(user.class_name, "advanced_class");
    // Should have deduplicated download_torrent and have upload_torrent
    assert!(user.permissions.contains(&UserPermission::DownloadTorrent));
    assert!(user.permissions.contains(&UserPermission::UploadTorrent));
    // Verify no duplicates by checking count
    let download_count = user
        .permissions
        .iter()
        .filter(|&p| p == &UserPermission::DownloadTorrent)
        .count();
    assert_eq!(
        download_count, 1,
        "DownloadTorrent should appear exactly once"
    );
    assert_eq!(user.permissions.len(), 2);
}

// ============================================================================
// USERS WITH STATS TESTS
// ============================================================================

#[sqlx::test(
    fixtures("with_test_users", "with_locked_user"),
    migrations = "../storage/migrations"
)]
async fn test_get_users_with_stats_filters_and_paginates(pool: PgPool) {
    let pool = ConnectionPool::with_pg_pool(pool);
    let newbie_class = vec![String::from("newbie")];

    let users = pool
        .get_users_with_stats(100, 0, &newbie_class)
        .await
        .expect("Failed to fetch users with stats");

    assert!(!users.is_empty());
    assert!(users.iter().all(|user| user.class_name == "newbie"));
    // the locked user must not be considered for a class change
    assert!(!users.iter().any(|user| user.id == 999));

    let users_of_unused_class = pool
        .get_users_with_stats(100, 0, &[String::from("unused_class")])
        .await
        .expect("Failed to fetch users with stats");
    assert!(users_of_unused_class.is_empty());

    let first_page = pool
        .get_users_with_stats(1, 0, &newbie_class)
        .await
        .expect("Failed to fetch users with stats");
    assert_eq!(first_page.len(), 1);
    let second_page = pool
        .get_users_with_stats(1, first_page[0].id, &newbie_class)
        .await
        .expect("Failed to fetch users with stats");
    assert_eq!(second_page.len(), 1);
    assert!(second_page[0].id > first_page[0].id);
}
