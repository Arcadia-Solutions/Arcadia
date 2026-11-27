use crate::{common, mocks};
use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::forum::{
    EditedForumSubCategory, EditedUserClassForbiddenForumItems, ForumItemId, ForumSearchResult,
    ForumThreadSortBy, ForumThreadSortDirection, UserClassForbiddenForumItems,
};
use common::{TestUser, auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

fn get_forum_thread_103(token: &str) -> actix_http::Request {
    test::TestRequest::get()
        .uri("/api/forum/thread?id=103")
        .insert_header(auth_header(token))
        .to_request()
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category",
    "with_test_forum_thread",
    "with_test_forum_post",
    "with_test_forum_access_restrictions"
))]
async fn test_hidden_forum_items_are_not_listed_nor_reachable(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/forum")
        .insert_header(auth_header(&user.token))
        .to_request();
    let overview: Value =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let test_category = overview["forum_categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|category| category["id"] == 100)
        .unwrap();
    let sub_category_ids: Vec<&Value> = test_category["sub_categories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|sub_category| &sub_category["id"])
        .collect();
    assert_eq!(sub_category_ids, [100]);

    let req = test::TestRequest::get()
        .uri("/api/search/forum?page=1&page_size=10&thread_name=")
        .insert_header(auth_header(&user.token))
        .to_request();
    let search: PaginatedResults<ForumSearchResult> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let mut thread_ids: Vec<i64> = search.results.iter().map(|r| r.thread_id).collect();
    thread_ids.sort_unstable();
    // thread 1 is the welcome thread of the initial schema
    assert_eq!(thread_ids, [1, 100, 101, 102]);
    assert_eq!(search.total_items, 4);

    // thread 103 is hidden through its sub category
    let resp = test::call_service(&service, get_forum_thread_103(&user.token)).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let req = test::TestRequest::post()
        .uri("/api/forum/post")
        .insert_header(auth_header(&user.token))
        .set_json(serde_json::json!({"content": "hello", "forum_thread_id": 103}))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

fn edited_sub_category_101(allowed_user_ids: Vec<i32>) -> EditedForumSubCategory {
    EditedForumSubCategory {
        id: 101,
        name: "Test Sub Category 2".into(),
        thread_sort_by: ForumThreadSortBy::LatestPost,
        thread_sort_direction: ForumThreadSortDirection::Descending,
        allowed_user_ids,
        new_threads_restricted: false,
    }
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category",
    "with_test_forum_thread",
    "with_test_forum_post",
    "with_test_forum_access_restrictions"
))]
async fn test_editing_allowed_users_of_a_forum_item(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, editor) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditForumSubCategory,
    )
    .await;

    // removing themselves would hide the sub category from the editor
    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&editor.token))
        .set_json(edited_sub_category_101(vec![]))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&editor.token))
        .set_json(edited_sub_category_101(vec![110, 999_999]))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // a failing edit leaves the allowed users untouched
    let mut edit_with_empty_name = edited_sub_category_101(vec![110, 100]);
    edit_with_empty_name.name = " ".into();
    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&editor.token))
        .set_json(edit_with_empty_name)
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let allowed_users = pool
        .find_forum_item_allowed_users(ForumItemId::SubCategory(101))
        .await
        .unwrap();
    assert_eq!(allowed_users.len(), 1);
    assert_eq!(allowed_users[0].id, 110);

    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&editor.token))
        .set_json(edited_sub_category_101(vec![110, 100]))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let req = test::TestRequest::get()
        .uri("/api/forum/sub-category?id=101")
        .insert_header(auth_header(&editor.token))
        .to_request();
    let sub_category: Value =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(
        sub_category["class_visibility"]["hidden_from_class_names"],
        serde_json::json!(["newbie"])
    );

    // user_basic now sees thread 103, but not who else is allowed in its sub category
    let (service, user) =
        create_test_app_and_login(pool.clone(), MockRedisPool::default(), TestUser::Standard).await;
    let resp = test::call_service(&service, get_forum_thread_103(&user.token)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let req = test::TestRequest::get()
        .uri("/api/forum/sub-category?id=101")
        .insert_header(auth_header(&user.token))
        .to_request();
    let sub_category: Value =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert!(sub_category["allowed_users"].is_null());
    assert!(sub_category["class_visibility"].is_null());

    // an editor who can't see the sub category can't change who is allowed in it
    let (service, editor_without_access) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ForumSubCategoryFlow,
    )
    .await;
    let req = test::TestRequest::put()
        .uri("/api/forum/sub-category")
        .insert_header(auth_header(&editor_without_access.token))
        .set_json(edited_sub_category_101(vec![]))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_forum_category",
    "with_test_forum_sub_category",
    "with_test_forum_thread",
    "with_test_forum_post",
    "with_test_forum_access_restrictions"
))]
async fn test_editing_forbidden_forum_items_of_a_user_class(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, editor) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditUserClass,
    )
    .await;

    // the editor is a newbie, but sees the items hidden from newbies since they can edit user classes
    let req = test::TestRequest::get()
        .uri("/api/user-classes/newbie/forbidden-forum-items")
        .insert_header(auth_header(&editor.token))
        .to_request();
    let forbidden: UserClassForbiddenForumItems =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let forbidden_sub_category_ids: Vec<i32> = forbidden
        .sub_categories
        .iter()
        .map(|sub_category| sub_category.id)
        .collect();
    assert_eq!(forbidden_sub_category_ids, [101]);

    let req = test::TestRequest::get()
        .uri("/api/user-classes/unknown/forbidden-forum-items")
        .insert_header(auth_header(&editor.token))
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let req = test::TestRequest::put()
        .uri("/api/user-classes/newbie/forbidden-forum-items")
        .insert_header(auth_header(&editor.token))
        .set_json(EditedUserClassForbiddenForumItems {
            category_ids: vec![],
            sub_category_ids: vec![999_999],
        })
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // hiding category 100 from newbies, its sub category 101 too
    let req = test::TestRequest::put()
        .uri("/api/user-classes/newbie/forbidden-forum-items")
        .insert_header(auth_header(&editor.token))
        .set_json(EditedUserClassForbiddenForumItems {
            category_ids: vec![100],
            sub_category_ids: vec![101],
        })
        .to_request();
    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = test::call_service(&service, get_forum_thread_103(&editor.token)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // being allowed in sub category 101 doesn't override its hidden category
    let (service, allowed_user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::EditForumSubCategory,
    )
    .await;
    let resp = test::call_service(&service, get_forum_thread_103(&allowed_user.token)).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
