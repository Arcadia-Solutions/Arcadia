pub mod common;
pub mod mocks;

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::common::PaginatedResults;
use arcadia_storage::models::forum::ForumPostHierarchy;
use arcadia_storage::models::reaction::ForumPostReactionUsers;
use common::{
    Profile, TestUser, auth_header, call_and_read_body_json, create_test_app_and_login, login_as,
};
use mocks::mock_redis::MockRedisPool;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis"
    ),
    migrations = "../storage/migrations"
)]
async fn test_react_twice_is_idempotent_and_unreact_removes(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ReactToContent,
    )
    .await;
    let user_id = current_user_id(&service, &user.token).await;

    for _ in 0..2 {
        let req = test::TestRequest::post()
            .uri("/api/forum/post/reaction")
            .insert_header(auth_header(&user.token))
            .set_json(json!({"forum_post_id": 100, "emoji_id": 100}))
            .to_request();
        let response = test::call_service(&service, req).await;
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let reactions = pool
        .find_reactions_for_forum_posts(&[100], user_id)
        .await
        .unwrap();
    let post_reactions = reactions.get(&100).expect("the post has reactions");
    assert_eq!(post_reactions.len(), 1);
    assert_eq!(post_reactions[0].amount, 1);
    assert!(post_reactions[0].reacted_by_current_user);

    let req = test::TestRequest::delete()
        .uri("/api/forum/post/reaction?forum_post_id=100&emoji_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let reactions = pool
        .find_reactions_for_forum_posts(&[100], user_id)
        .await
        .unwrap();
    assert!(!reactions.contains_key(&100));

    // Unreacting again matches no row: reported rather than silently succeeding.
    let req = test::TestRequest::delete()
        .uri("/api/forum/post/reaction?forum_post_id=100&emoji_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis"
    ),
    migrations = "../storage/migrations"
)]
async fn test_several_emojis_and_unknown_emoji(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ReactToContent,
    )
    .await;
    let user_id = current_user_id(&service, &user.token).await;

    for emoji_id in [100, 101] {
        let req = test::TestRequest::post()
            .uri("/api/forum/post/reaction")
            .insert_header(auth_header(&user.token))
            .set_json(json!({"forum_post_id": 100, "emoji_id": emoji_id}))
            .to_request();
        let response = test::call_service(&service, req).await;
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let reactions = pool
        .find_reactions_for_forum_posts(&[100], user_id)
        .await
        .unwrap();
    assert_eq!(reactions.get(&100).unwrap().len(), 2);

    let req = test::TestRequest::post()
        .uri("/api/forum/post/reaction")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"forum_post_id": 100, "emoji_id": 999}))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis"
    ),
    migrations = "../storage/migrations"
)]
async fn test_user_without_permission_cannot_react(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::post()
        .uri("/api/forum/post/reaction")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"forum_post_id": 100, "emoji_id": 100}))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let req = test::TestRequest::delete()
        .uri("/api/forum/post/reaction?forum_post_id=100&emoji_id=100")
        .insert_header(auth_header(&user.token))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis"
    ),
    migrations = "../storage/migrations"
)]
async fn test_deleting_a_post_deletes_its_reactions(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ReactToContent,
    )
    .await;
    let user_id = current_user_id(&service, &user.token).await;

    let req = test::TestRequest::post()
        .uri("/api/forum/post/reaction")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"forum_post_id": 100, "emoji_id": 100}))
        .to_request();
    test::call_service(&service, req).await;

    pool.delete_forum_post(100).await.unwrap();

    let reactions = pool
        .find_reactions_for_forum_posts(&[100], user_id)
        .await
        .unwrap();
    assert!(reactions.is_empty());
}

/// `LoginResponse` only carries the authentication tokens, so the database id of the logged
/// in user is read from their profile instead.
async fn current_user_id<S>(service: &S, token: &str) -> i32
where
    S: actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
{
    let req = test::TestRequest::get()
        .uri("/api/users/me")
        .insert_header(auth_header(token))
        .to_request();
    let profile = call_and_read_body_json::<Profile, _>(service, req).await;
    profile.user.id
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis"
    ),
    migrations = "../storage/migrations"
)]
async fn test_thread_posts_carry_their_reactions(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ReactToContent).await;

    let req = test::TestRequest::post()
        .uri("/api/forum/post/reaction")
        .insert_header(auth_header(&user.token))
        .set_json(json!({"forum_post_id": 100, "emoji_id": 100}))
        .to_request();
    test::call_service(&service, req).await;

    let req = test::TestRequest::get()
        .uri("/api/forum/thread/posts?thread_id=100&page=1&page_size=10")
        .insert_header(auth_header(&user.token))
        .to_request();
    let posts: PaginatedResults<ForumPostHierarchy> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    let post = posts
        .results
        .iter()
        .find(|post| post.id == 100)
        .expect("the post is on the page");
    assert_eq!(post.reactions.len(), 1);
    assert_eq!(post.reactions[0].emoji_id, 100);
    assert_eq!(post.reactions[0].amount, 1);
    assert_eq!(
        post.reactions[0].emoji_unicode_character.as_deref(),
        Some("👍")
    );
    assert!(post.reactions[0].reacted_by_current_user);

    // A post nobody reacted to carries an empty list, never a null.
    let post_without_reactions = posts.results.iter().find(|post| post.id != 100);
    if let Some(post) = post_without_reactions {
        assert!(post.reactions.is_empty());
    }
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis"
    ),
    migrations = "../storage/migrations"
)]
async fn test_reaction_users_are_grouped_by_emoji(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, reactor) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ReactToContent,
    )
    .await;

    for emoji_id in [100, 101] {
        let req = test::TestRequest::post()
            .uri("/api/forum/post/reaction")
            .insert_header(auth_header(&reactor.token))
            .set_json(json!({"forum_post_id": 100, "emoji_id": emoji_id}))
            .to_request();
        test::call_service(&service, req).await;
    }

    let req = test::TestRequest::get()
        .uri("/api/forum/post/reaction/users?forum_post_id=100")
        .insert_header(auth_header(&reactor.token))
        .to_request();
    let grouped: Vec<ForumPostReactionUsers> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(grouped.len(), 2);
    for group in &grouped {
        assert_eq!(group.total_amount, 1);
        assert_eq!(group.users.len(), 1);
        assert_eq!(group.users[0].username, "user_reactor");
    }

    // A post without reactions returns an empty list rather than an error.
    let req = test::TestRequest::get()
        .uri("/api/forum/post/reaction/users?forum_post_id=101")
        .insert_header(auth_header(&reactor.token))
        .to_request();
    let grouped: Vec<ForumPostReactionUsers> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    assert!(grouped.is_empty());
}

/// Two emojis that share the same `sort_order` must still keep each emoji's reactors
/// contiguous in the response: the ordering must fall back to `emoji_id` so that grouping
/// by "the last group" (which relies on contiguous rows per emoji) stays correct.
#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis_tied_sort_order",
        "with_test_forum_post_reactions_tied_sort_order"
    ),
    migrations = "../storage/migrations"
)]
async fn test_reaction_users_stay_grouped_when_emojis_share_a_sort_order(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, reactor) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::ReactToContent).await;

    let req = test::TestRequest::get()
        .uri("/api/forum/post/reaction/users?forum_post_id=100")
        .insert_header(auth_header(&reactor.token))
        .to_request();
    let grouped: Vec<ForumPostReactionUsers> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    assert_eq!(
        grouped.len(),
        2,
        "one group per emoji, not one per reaction"
    );

    let emoji_100 = grouped
        .iter()
        .find(|group| group.emoji_id == 100)
        .expect("emoji 100 has a group");
    assert_eq!(emoji_100.total_amount, 2);
    let usernames: Vec<&str> = emoji_100
        .users
        .iter()
        .map(|user| user.username.as_str())
        .collect();
    assert_eq!(usernames, vec!["user_basic", "user_edit_art"]);

    let emoji_101 = grouped
        .iter()
        .find(|group| group.emoji_id == 101)
        .expect("emoji 101 has a group");
    assert_eq!(emoji_101.total_amount, 3);
    let usernames: Vec<&str> = emoji_101
        .users
        .iter()
        .map(|user| user.username.as_str())
        .collect();
    assert_eq!(
        usernames,
        vec!["user_edit_ser", "user_edit_tgc", "user_css_crt"]
    );
}

/// The positive case (a user who reacted seeing `reacted_by_current_user: true`) is already
/// covered elsewhere; this pins the other side, which the highlighted chip on the frontend
/// depends on just as much.
#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis"
    ),
    migrations = "../storage/migrations"
)]
async fn test_reacted_by_current_user_is_false_for_a_user_who_did_not_react(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, reactor) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ReactToContent,
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/forum/post/reaction")
        .insert_header(auth_header(&reactor.token))
        .set_json(json!({"forum_post_id": 100, "emoji_id": 100}))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let bystander = login_as(&service, TestUser::Standard).await;
    let bystander_id = current_user_id(&service, &bystander.token).await;

    let reactions = pool
        .find_reactions_for_forum_posts(&[100], bystander_id)
        .await
        .unwrap();
    let post_reactions = reactions.get(&100).expect("the post has reactions");
    assert_eq!(post_reactions.len(), 1);
    assert_eq!(post_reactions[0].amount, 1);
    assert!(!post_reactions[0].reacted_by_current_user);
}

#[derive(Debug, serde::Deserialize)]
struct ErrorResponse {
    error: String,
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis"
    ),
    migrations = "../storage/migrations"
)]
async fn test_reacting_with_a_disabled_emoji_is_rejected(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, staff) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::EditArcadiaSettings,
    )
    .await;

    let req = test::TestRequest::put()
        .uri("/api/emojis/enabled")
        .insert_header(auth_header(&staff.token))
        .set_json(json!({"id": 100, "enabled": false}))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let reactor = login_as(&service, TestUser::ReactToContent).await;
    let req = test::TestRequest::post()
        .uri("/api/forum/post/reaction")
        .insert_header(auth_header(&reactor.token))
        .set_json(json!({"forum_post_id": 100, "emoji_id": 100}))
        .to_request();
    let error: ErrorResponse =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::BAD_REQUEST).await;
    assert_eq!(error.error, "this emoji is disabled");
}

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_forum_category",
        "with_test_forum_sub_category",
        "with_test_forum_thread",
        "with_test_forum_post",
        "with_test_emojis"
    ),
    migrations = "../storage/migrations"
)]
async fn test_existing_reaction_on_a_disabled_emoji_is_still_returned(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, reactor) = create_test_app_and_login(
        pool.clone(),
        MockRedisPool::default(),
        TestUser::ReactToContent,
    )
    .await;
    let user_id = current_user_id(&service, &reactor.token).await;

    // React while the emoji is still enabled.
    let req = test::TestRequest::post()
        .uri("/api/forum/post/reaction")
        .insert_header(auth_header(&reactor.token))
        .set_json(json!({"forum_post_id": 100, "emoji_id": 100}))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    // Disabling the emoji afterwards must not erase the history of who reacted with it.
    let staff = login_as(&service, TestUser::EditArcadiaSettings).await;
    let req = test::TestRequest::put()
        .uri("/api/emojis/enabled")
        .insert_header(auth_header(&staff.token))
        .set_json(json!({"id": 100, "enabled": false}))
        .to_request();
    let response = test::call_service(&service, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    let reactions = pool
        .find_reactions_for_forum_posts(&[100], user_id)
        .await
        .unwrap();
    let post_reactions = reactions.get(&100).expect("the reaction is still there");
    assert_eq!(post_reactions.len(), 1);
    assert_eq!(post_reactions[0].emoji_id, 100);
    assert_eq!(post_reactions[0].amount, 1);

    // It must also still be carried alongside the thread's posts.
    let req = test::TestRequest::get()
        .uri("/api/forum/thread/posts?thread_id=100&page=1&page_size=10")
        .insert_header(auth_header(&reactor.token))
        .to_request();
    let posts: PaginatedResults<ForumPostHierarchy> =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;
    let post = posts
        .results
        .iter()
        .find(|post| post.id == 100)
        .expect("the post is on the page");
    assert_eq!(post.reactions.len(), 1);
    assert_eq!(post.reactions[0].emoji_id, 100);
}
