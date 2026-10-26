pub mod common;
pub mod mocks;

use crate::common::TestUser;
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::title_group_tag::{
    DeleteTitleGroupTagRequest, TitleGroupAppliedTag, TitleGroupTag, TitleGroupTagEnriched,
    TitleGroupTagLite,
};
use common::{auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_tag"),
    migrations = "../storage/migrations"
)]
async fn test_soft_delete_tag_then_recreate_is_rejected(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Step 1: delete the tag (soft delete)
    let (service, delete_user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageTitleGroupTags,
    )
    .await;

    let delete_body = DeleteTitleGroupTagRequest {
        id: 1,
        deletion_reason: "duplicate of adventure".into(),
    };

    let req = test::TestRequest::delete()
        .uri("/api/title-group-tags")
        .insert_header(auth_header(&delete_user.token))
        .set_json(&delete_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Step 2: verify the tag no longer appears in search results
    let req = test::TestRequest::get()
        .uri("/api/search/title-group-tags/lite?name=action&page=1&page_size=10")
        .insert_header(auth_header(&delete_user.token))
        .to_request();

    let response: PaginatedResults<TitleGroupTagLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.results.len(), 0);

    // Step 3: attempt to recreate the same tag — should fail with the deletion reason
    let (service, standard_user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .uri("/api/title-group-tags")
        .insert_header(auth_header(&standard_user.token))
        .set_json(serde_json::json!({"name": "action"}))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_tag"),
    migrations = "../storage/migrations"
)]
async fn test_create_tag_returns_existing_non_deleted_tag(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .uri("/api/title-group-tags")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({"name": "action"}))
        .to_request();

    let response: TitleGroupTag =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    assert_eq!(response.name, "action");
    assert_eq!(response.id, 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_tag",
        "with_test_title_group_tag_applied"
    ),
    migrations = "../storage/migrations"
)]
async fn test_delete_tag_removes_it_from_all_title_groups(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    // Verify the tag is initially applied to the title group
    let title_group = pool.find_title_group(1).await.unwrap();
    assert!(title_group.tags.contains(&"action".to_string()));

    // Delete the tag
    let (service, delete_user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageTitleGroupTags,
    )
    .await;

    let delete_body = DeleteTitleGroupTagRequest {
        id: 1,
        deletion_reason: "no longer needed".into(),
    };

    let req = test::TestRequest::delete()
        .uri("/api/title-group-tags")
        .insert_header(auth_header(&delete_user.token))
        .set_json(&delete_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the tag has been removed from the title group
    let title_group = pool.find_title_group(1).await.unwrap();
    assert!(title_group.tags.is_empty());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_tag"),
    migrations = "../storage/migrations"
)]
async fn test_searching_deleted_tags(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, delete_user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageTitleGroupTags,
    )
    .await;

    let delete_body = DeleteTitleGroupTagRequest {
        id: 1,
        deletion_reason: "duplicate of adventure".into(),
    };

    let req = test::TestRequest::delete()
        .uri("/api/title-group-tags")
        .insert_header(auth_header(&delete_user.token))
        .set_json(&delete_body)
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let search_tags = async |show_deleted: bool| {
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/search/title-group-tags?name=action&page=1&page_size=10&order_by_column=name&order_by_direction=asc&show_deleted={show_deleted}"
            ))
            .insert_header(auth_header(&delete_user.token))
            .to_request();

        common::call_and_read_body_json_with_status::<PaginatedResults<TitleGroupTagEnriched>, _>(
            &service,
            req,
            StatusCode::OK,
        )
        .await
    };

    let live_tags = search_tags(false).await;
    assert_eq!(live_tags.results.len(), 0);

    let all_tags = search_tags(true).await;
    assert_eq!(all_tags.results.len(), 1);

    let deleted_tag = &all_tags.results[0];
    assert!(deleted_tag.deleted_at.is_some());
    assert_eq!(
        deleted_tag.deletion_reason.as_deref(),
        Some("duplicate of adventure")
    );
    assert!(deleted_tag.deleted_by.is_some());
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_tag"),
    migrations = "../storage/migrations"
)]
async fn test_restoring_deleted_tag(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, manage_user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageTitleGroupTags,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/title-group-tags")
        .insert_header(auth_header(&manage_user.token))
        .set_json(&DeleteTitleGroupTagRequest {
            id: 1,
            deletion_reason: "deleted by mistake".into(),
        })
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::put()
        .uri("/api/title-group-tags/restore")
        .insert_header(auth_header(&manage_user.token))
        .set_json(serde_json::json!({"id": 1}))
        .to_request();

    let restored_tag: TitleGroupTag =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(restored_tag.name, "action");

    // the tag is searchable again
    let req = test::TestRequest::get()
        .uri("/api/search/title-group-tags/lite?name=action&page=1&page_size=10")
        .insert_header(auth_header(&manage_user.token))
        .to_request();

    let response: PaginatedResults<TitleGroupTagLite> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(response.results.len(), 1);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_tag",
        "with_test_title_group_tag_applied",
        "with_test_title_group_tag_merge_target"
    ),
    migrations = "../storage/migrations"
)]
async fn test_merging_tags(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, manage_user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageTitleGroupTags,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/title-group-tags/merge")
        .insert_header(auth_header(&manage_user.token))
        .set_json(serde_json::json!({"source_tag_id": 1, "target_tag_id": 2}))
        .to_request();

    let merged_tag: TitleGroupTag =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // the source tag became a synonym of the target one
    assert_eq!(merged_tag.synonyms, vec!["action".to_string()]);

    // the title groups of the source tag were moved onto the target one
    let title_group = pool.find_title_group(1).await.unwrap();
    assert_eq!(title_group.tags, vec!["action.movies".to_string()]);

    // the name of the source tag now resolves to the target tag
    let resolved_names = pool.resolve_tag_names(&["action".into()]).await.unwrap();
    assert_eq!(resolved_names, vec!["action.movies".to_string()]);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_tag"
    ),
    migrations = "../storage/migrations"
)]
async fn test_applying_deleted_tag_is_rejected(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, manage_user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageTitleGroupTags,
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/api/title-group-tags")
        .insert_header(auth_header(&manage_user.token))
        .set_json(&DeleteTitleGroupTagRequest {
            id: 1,
            deletion_reason: "not a genre".into(),
        })
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::post()
        .uri("/api/title-group-tags/apply")
        .insert_header(auth_header(&manage_user.token))
        .set_json(serde_json::json!({"title_group_id": 1, "tag_id": 1}))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_title_group_tag_synonyms"),
    migrations = "../storage/migrations"
)]
async fn test_resolving_scraped_tag_names(pool: PgPool) {
    let pool = ConnectionPool::with_pg_pool(pool);

    let resolved_names = pool
        .resolve_tag_names(&[
            "Science Fiction".into(), // sanitized into an existing tag
            "scifi".into(),           // synonym of that same tag
            "sci fi".into(),          // synonym once sanitized
            "Sci-Fi".into(),          // dashes separate words like spaces do
            "blu.ray".into(),         // deleted tag
            "Road Movie".into(),      // unknown tag
        ])
        .await
        .unwrap();

    assert_eq!(resolved_names, vec!["science.fiction", "road.movie"]);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_title_group",
        "with_test_title_group_tag",
        "with_test_title_group_tag_applied"
    ),
    migrations = "../storage/migrations"
)]
async fn test_search_title_group_applied_tags(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, manage_user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ManageTitleGroupTags,
    )
    .await;

    let search = |filters: &str| {
        test::TestRequest::get()
            .uri(&format!(
                "/api/title-group-tags/applied?page=1&page_size=10&order_by_column=title_group_tags_amount&order_by_direction=desc{filters}"
            ))
            .insert_header(auth_header(&manage_user.token))
            .to_request()
    };

    let results: PaginatedResults<TitleGroupAppliedTag> =
        common::call_and_read_body_json_with_status(
            &service,
            search("&tag_name=action"),
            StatusCode::OK,
        )
        .await;
    assert_eq!(results.total_items, 1);
    assert_eq!(results.results[0].tag_name, "action");
    assert_eq!(results.results[0].title_group.id, 1);
    assert_eq!(
        results.results[0].title_group.tags,
        vec!["action".to_string()]
    );
    assert_eq!(results.results[0].user_applied_tags_amount, 1);

    let results: PaginatedResults<TitleGroupAppliedTag> =
        common::call_and_read_body_json_with_status(
            &service,
            search("&tag_name=act"),
            StatusCode::OK,
        )
        .await;
    assert_eq!(results.total_items, 0);

    // the time range only scopes the per-user count, it does not filter the results
    let results: PaginatedResults<TitleGroupAppliedTag> =
        common::call_and_read_body_json_with_status(
            &service,
            search("&user_applied_tags_after=2999-01-01T00:00:00Z"),
            StatusCode::OK,
        )
        .await;
    assert_eq!(results.total_items, 1);
    assert_eq!(results.results[0].user_applied_tags_amount, 0);
}
