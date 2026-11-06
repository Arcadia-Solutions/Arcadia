use crate::{common, mocks};

use actix_web::http::StatusCode;
use actix_web::test;
use arcadia_storage::connection_pool::ConnectionPool;
use arcadia_storage::models::user_staff_note::{
    EditedUserStaffNote, UserCreatedStaffNote, UserStaffNoteWithAuthor,
};
use common::{TestUser, auth_header, create_test_app_and_login};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

const BASIC_USER_ID: i32 = 100;

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_staff_notes"),
    migrations = "../storage/migrations"
)]
async fn test_get_user_staff_notes_requires_write_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes"))
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_staff_notes"),
    migrations = "../storage/migrations"
)]
async fn test_note_writer_only_sees_own_notes(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::WriteUserStaffNote)
            .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes"))
        .insert_header(auth_header(&user.token))
        .to_request();

    let notes: Vec<UserStaffNoteWithAuthor> = common::call_and_read_body_json(&service, req).await;

    assert_eq!(notes.len(), 2);
    assert!(
        notes
            .iter()
            .all(|note| note.author_username == "user_wr_note")
    );
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_staff_notes"),
    migrations = "../storage/migrations"
)]
async fn test_viewer_of_foreign_notes_sees_every_note(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) = create_test_app_and_login(
        pool,
        MockRedisPool::default(),
        TestUser::ViewForeignUserStaffNotes,
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes"))
        .insert_header(auth_header(&user.token))
        .to_request();

    let notes: Vec<UserStaffNoteWithAuthor> = common::call_and_read_body_json(&service, req).await;

    assert_eq!(notes.len(), 3);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_staff_notes"),
    migrations = "../storage/migrations"
)]
async fn test_staff_can_create_note(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::WriteUserStaffNote)
            .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes"))
        .insert_header(auth_header(&user.token))
        .set_json(UserCreatedStaffNote {
            content: "A brand new note".to_string(),
        })
        .to_request();

    let note: UserStaffNoteWithAuthor =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::CREATED).await;

    assert_eq!(note.user_id, BASIC_USER_ID);
    assert_eq!(note.author_username, "user_wr_note");
    assert_eq!(note.content, "A brand new note");
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_staff_notes"),
    migrations = "../storage/migrations"
)]
async fn test_author_can_edit_and_delete_own_recent_note_without_permission(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::WriteUserStaffNote)
            .await;

    let edit_req = test::TestRequest::put()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes/1000"))
        .insert_header(auth_header(&user.token))
        .set_json(EditedUserStaffNote {
            content: "An edited note".to_string(),
        })
        .to_request();
    let resp = test::call_service(&service, edit_req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let notes_req = test::TestRequest::get()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes"))
        .insert_header(auth_header(&user.token))
        .to_request();
    let notes: Vec<UserStaffNoteWithAuthor> =
        common::call_and_read_body_json(&service, notes_req).await;
    assert_eq!(
        notes
            .iter()
            .find(|note| note.id == 1000)
            .expect("the edited note")
            .content,
        "An edited note"
    );

    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes/1000"))
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, delete_req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let remaining_req = test::TestRequest::get()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes"))
        .insert_header(auth_header(&user.token))
        .to_request();
    let remaining: Vec<UserStaffNoteWithAuthor> =
        common::call_and_read_body_json(&service, remaining_req).await;
    assert_eq!(remaining.len(), 1);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_staff_notes"),
    migrations = "../storage/migrations"
)]
async fn test_author_cannot_edit_note_older_than_the_window(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::WriteUserStaffNote)
            .await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes/1001"))
        .insert_header(auth_header(&user.token))
        .set_json(EditedUserStaffNote {
            content: "An edited note".to_string(),
        })
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_staff_notes"),
    migrations = "../storage/migrations"
)]
async fn test_note_writer_cannot_edit_note_of_another_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::WriteUserStaffNote)
            .await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes/1002"))
        .insert_header(auth_header(&user.token))
        .set_json(EditedUserStaffNote {
            content: "An edited note".to_string(),
        })
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_staff_notes"),
    migrations = "../storage/migrations"
)]
async fn test_permission_holder_can_edit_and_delete_old_note_of_another_user(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditUserStaffNotes)
            .await;

    let edit_req = test::TestRequest::put()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes/1001"))
        .insert_header(auth_header(&user.token))
        .set_json(EditedUserStaffNote {
            content: "An edited note".to_string(),
        })
        .to_request();
    let resp = test::call_service(&service, edit_req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let delete_req = test::TestRequest::delete()
        .uri(&format!("/api/users/{BASIC_USER_ID}/staff-notes/1001"))
        .insert_header(auth_header(&user.token))
        .to_request();
    let resp = test::call_service(&service, delete_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(
    fixtures("with_test_users", "with_test_user_staff_notes"),
    migrations = "../storage/migrations"
)]
async fn test_note_of_another_user_is_not_found(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::EditUserStaffNotes)
            .await;

    // the note 1000 belongs to the user 100, not to the user 192
    let req = test::TestRequest::delete()
        .uri("/api/users/192/staff-notes/1000")
        .insert_header(auth_header(&user.token))
        .to_request();

    let resp = test::call_service(&service, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
