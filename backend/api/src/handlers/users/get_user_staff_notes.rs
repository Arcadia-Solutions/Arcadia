use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{user::UserPermission, user_staff_note::UserStaffNoteWithAuthor},
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Get user staff notes",
    tag = "User",
    path = "/api/users/{id}/staff-notes",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Successfully got the user staff notes", body=Vec<UserStaffNoteWithAuthor>),
        (status = 403, description = "Insufficient permissions"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user_id: Path<i32>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            current_user.sub,
            &UserPermission::WriteUserStaffNote,
            req.path(),
        )
        .await?;

    // only the users with the permission can read the notes of the other members of the staff,
    // the other ones only get the notes they wrote themselves
    let can_view_foreign_notes = arc
        .pool
        .user_has_permission(current_user.sub, &UserPermission::ViewForeignUserStaffNotes)
        .await?;

    let staff_notes = arc
        .pool
        .find_user_staff_notes(*user_id, can_view_foreign_notes, current_user.sub)
        .await?;

    Ok(HttpResponse::Ok().json(staff_notes))
}
