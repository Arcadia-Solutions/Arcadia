use crate::{common, mocks};

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::{
    common::PaginatedResults,
    title_group::{TitleGroupAndAssociatedData, TitleGroupHierarchyLite},
    title_group_comment::UserCreatedTitleGroupComment,
    title_group_review::{
        EditedTitleGroupReview, TitleGroupReview, TitleGroupReviewHierarchy,
        TitleGroupReviewWithTitleGroup, UserCreatedTitleGroupReview,
    },
};
use common::{TestUser, auth_header, create_test_app_and_login, login_as};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

fn create_review_request(token: &str, title_group_id: i32, rating: i16) -> actix_http::Request {
    test::TestRequest::post()
        .uri("/api/title-groups/reviews")
        .insert_header(auth_header(token))
        .set_json(UserCreatedTitleGroupReview {
            title_group_id,
            rating,
            review: Some("[b]good[/b]".into()),
        })
        .to_request()
}

fn get_title_group_request(token: &str, title_group_id: i32) -> actix_http::Request {
    test::TestRequest::get()
        .uri(&format!("/api/title-groups?id={title_group_id}"))
        .insert_header(auth_header(token))
        .to_request()
}

fn get_title_group_reviews_request(token: &str, title_group_id: i32) -> actix_http::Request {
    test::TestRequest::get()
        .uri(&format!(
            "/api/title-groups/reviews?title_group_id={title_group_id}"
        ))
        .insert_header(auth_header(token))
        .to_request()
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_create_and_edit_review_updates_aggregates(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let review: TitleGroupReview = common::call_and_read_body_json_with_status(
        &service,
        create_review_request(&user.token, 1, 80),
        StatusCode::CREATED,
    )
    .await;

    // one review per user per title group
    let response = test::call_service(&service, create_review_request(&user.token, 1, 50)).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let response = test::call_service(&service, create_review_request(&user.token, 2, 101)).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let req = test::TestRequest::put()
        .uri(&format!("/api/title-groups/reviews/{}", review.id))
        .insert_header(auth_header(&user.token))
        .set_json(EditedTitleGroupReview {
            rating: 60,
            review: None,
        })
        .to_request();
    let edited: TitleGroupReview =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(edited.rating, 60);

    let req = test::TestRequest::post()
        .uri("/api/title-groups/comments")
        .insert_header(auth_header(&user.token))
        .set_json(UserCreatedTitleGroupComment {
            content: "nice".into(),
            title_group_id: 1,
            refers_to_torrent_id: None,
            answers_to_comment_id: None,
        })
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let title_group: TitleGroupAndAssociatedData = common::call_and_read_body_json_with_status(
        &service,
        get_title_group_request(&user.token, 1),
        StatusCode::OK,
    )
    .await;
    assert_eq!(title_group.title_group.user_rating_average, Some(60.0));
    assert_eq!(title_group.title_group.user_rating_count, 1);
    assert_eq!(title_group.title_group_comments[0].author_rating, Some(60));

    let reviews: Vec<TitleGroupReviewHierarchy> = common::call_and_read_body_json_with_status(
        &service,
        get_title_group_reviews_request(&user.token, 1),
        StatusCode::OK,
    )
    .await;
    assert_eq!(reviews.len(), 1);
    assert_eq!(reviews[0].id, review.id);

    let req = test::TestRequest::get()
        .uri("/api/search/title-group-reviews/user?user_id=100&page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();
    let user_reviews: PaginatedResults<TitleGroupReviewWithTitleGroup> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(user_reviews.total_items, 1);
    assert_eq!(user_reviews.results[0].title_group_id, 1);
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_only_moderators_can_delete_reviews(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let review: TitleGroupReview = common::call_and_read_body_json_with_status(
        &service,
        create_review_request(&user.token, 1, 80),
        StatusCode::CREATED,
    )
    .await;
    let delete_request = |token: &str| {
        test::TestRequest::delete()
            .uri(&format!("/api/title-groups/reviews/{}", review.id))
            .insert_header(auth_header(token))
            .to_request()
    };

    let response = test::call_service(&service, delete_request(&user.token)).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let moderator = login_as(&service, TestUser::DeleteTitleGroupReview).await;
    let response = test::call_service(&service, delete_request(&moderator.token)).await;
    assert_eq!(response.status(), StatusCode::OK);

    let title_group: TitleGroupAndAssociatedData = common::call_and_read_body_json_with_status(
        &service,
        get_title_group_request(&user.token, 1),
        StatusCode::OK,
    )
    .await;
    assert_eq!(title_group.title_group.user_rating_average, None);
    assert_eq!(title_group.title_group.user_rating_count, 0);

    let reviews: Vec<TitleGroupReviewHierarchy> = common::call_and_read_body_json_with_status(
        &service,
        get_title_group_reviews_request(&user.token, 1),
        StatusCode::OK,
    )
    .await;
    assert!(reviews.is_empty());
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_title_group_reviews_requiring_snatch_or_seeding"
))]
async fn test_review_requires_snatch_or_seeding_when_enabled(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let response = test::call_service(&service, create_review_request(&user.token, 2, 80)).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let response = test::call_service(&service, create_review_request(&user.token, 1, 80)).await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_title_group_reviews",
    "with_refreshed_title_group_hierarchy_lite"
))]
async fn test_search_torrents_sorts_by_rating(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let search = |parameters: &str| {
        test::TestRequest::get()
            .uri(&format!(
                "/api/search/torrents/lite?page=1&page_size=10&title_group_include_empty_groups=true&order_by_column=title_group_user_rating_average&{parameters}"
            ))
            .insert_header(auth_header(&user.token))
            .to_request()
    };

    let results: PaginatedResults<TitleGroupHierarchyLite> =
        common::call_and_read_body_json_with_status(
            &service,
            search("order_by_direction=asc"),
            StatusCode::OK,
        )
        .await;
    let ids: Vec<i32> = results.results.iter().map(|group| group.id).collect();
    assert_eq!(ids, vec![2, 1]);
    assert_eq!(results.results[1].user_rating_average, Some(80.0));
    assert_eq!(results.results[1].user_rating_count, 1);
}
