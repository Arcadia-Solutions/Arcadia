use crate::{common, mocks};

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::{
    common::PaginatedResults,
    title_group::{TitleGroupAndAssociatedData, TitleGroupHierarchyLite},
    title_group_comment::UserCreatedTitleGroupComment,
    title_group_rating::{
        EditedTitleGroupRating, TitleGroupRating, TitleGroupRatingHierarchy,
        TitleGroupRatingWithTitleGroup, UserCreatedTitleGroupRating,
    },
};
use common::{TestUser, auth_header, create_test_app_and_login, login_as};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

fn create_rating_request(token: &str, title_group_id: i32, rating: i16) -> actix_http::Request {
    test::TestRequest::post()
        .uri("/api/title-groups/ratings")
        .insert_header(auth_header(token))
        .set_json(UserCreatedTitleGroupRating {
            title_group_id,
            rating,
            comment: Some("[b]good[/b]".into()),
        })
        .to_request()
}

fn get_title_group_request(token: &str, title_group_id: i32) -> actix_http::Request {
    test::TestRequest::get()
        .uri(&format!("/api/title-groups?id={title_group_id}"))
        .insert_header(auth_header(token))
        .to_request()
}

fn search_ratings_request(token: &str, parameters: &str) -> actix_http::Request {
    test::TestRequest::get()
        .uri(&format!(
            "/api/search/title-group-ratings?page=1&page_size=10&order_by_column=created_at&order_by_direction=desc&{parameters}"
        ))
        .insert_header(auth_header(token))
        .to_request()
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_create_and_edit_rating_updates_aggregates(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let rating: TitleGroupRating = common::call_and_read_body_json_with_status(
        &service,
        create_rating_request(&user.token, 1, 80),
        StatusCode::CREATED,
    )
    .await;

    // one rating per user per title group
    let response = test::call_service(&service, create_rating_request(&user.token, 1, 50)).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let response = test::call_service(&service, create_rating_request(&user.token, 2, 101)).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let req = test::TestRequest::put()
        .uri(&format!("/api/title-groups/ratings/{}", rating.id))
        .insert_header(auth_header(&user.token))
        .set_json(EditedTitleGroupRating {
            rating: 60,
            comment: None,
        })
        .to_request();
    let edited: TitleGroupRating =
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
    assert_eq!(title_group.own_rating.map(|rating| rating.rating), Some(60));

    let ratings: PaginatedResults<TitleGroupRatingHierarchy> =
        common::call_and_read_body_json_with_status(
            &service,
            search_ratings_request(
                &user.token,
                "title_group_id=1&minimum_rating=0&maximum_rating=100&content=",
            ),
            StatusCode::OK,
        )
        .await;
    assert_eq!(ratings.results.len(), 1);
    assert_eq!(ratings.results[0].id, rating.id);

    let req = test::TestRequest::get()
        .uri("/api/search/title-group-ratings/user?user_id=100&minimum_rating=0&maximum_rating=100&content=&order_by_column=created_at&order_by_direction=desc&page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();
    let user_ratings: PaginatedResults<TitleGroupRatingWithTitleGroup> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert_eq!(user_ratings.total_items, 1);
    assert_eq!(user_ratings.results[0].title_group.id, 1);
}

#[sqlx::test(fixtures("with_test_users", "with_test_title_group"))]
async fn test_only_moderators_can_delete_ratings(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let rating: TitleGroupRating = common::call_and_read_body_json_with_status(
        &service,
        create_rating_request(&user.token, 1, 80),
        StatusCode::CREATED,
    )
    .await;
    let delete_request = |token: &str| {
        test::TestRequest::delete()
            .uri(&format!("/api/title-groups/ratings/{}", rating.id))
            .insert_header(auth_header(token))
            .to_request()
    };

    let response = test::call_service(&service, delete_request(&user.token)).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let moderator = login_as(&service, TestUser::DeleteTitleGroupRating).await;
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

    let ratings: PaginatedResults<TitleGroupRatingHierarchy> =
        common::call_and_read_body_json_with_status(
            &service,
            search_ratings_request(
                &user.token,
                "title_group_id=1&minimum_rating=0&maximum_rating=100&content=",
            ),
            StatusCode::OK,
        )
        .await;
    assert_eq!(ratings.total_items, 0);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_title_group_ratings"
))]
async fn test_search_ratings_filters_by_rating_and_content(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    // title group 1 has a single rating, 80 with the comment "Great"
    for (parameters, expected_total_items) in [
        (
            "title_group_id=1&minimum_rating=80&maximum_rating=80&content=gReAt",
            1,
        ),
        (
            "title_group_id=1&minimum_rating=81&maximum_rating=100&content=",
            0,
        ),
        (
            "title_group_id=1&minimum_rating=0&maximum_rating=79&content=",
            0,
        ),
        (
            "title_group_id=1&minimum_rating=0&maximum_rating=100&content=bad",
            0,
        ),
    ] {
        let ratings: PaginatedResults<TitleGroupRatingHierarchy> =
            common::call_and_read_body_json_with_status(
                &service,
                search_ratings_request(&user.token, parameters),
                StatusCode::OK,
            )
            .await;
        assert_eq!(ratings.total_items, expected_total_items, "{parameters}");
    }

    let response = test::call_service(
        &service,
        search_ratings_request(
            &user.token,
            "title_group_id=1&minimum_rating=60&maximum_rating=40&content=",
        ),
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_edition_group",
    "with_test_torrent",
    "with_title_group_ratings_requiring_snatch_or_seeding"
))]
async fn test_rating_requires_snatch_or_seeding_when_enabled(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let response = test::call_service(&service, create_rating_request(&user.token, 2, 80)).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let response = test::call_service(&service, create_rating_request(&user.token, 1, 80)).await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_title_group_ratings",
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
