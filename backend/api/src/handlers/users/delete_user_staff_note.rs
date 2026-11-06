use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[utoipa::path(
    delete,
    operation_id = "Delete user staff note",
    tag = "User",
    path = "/api/users/{id}/staff-notes/{staff_note_id}",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID"),
        ("staff_note_id" = i64, Path, description = "Staff note ID")
    ),
    responses(
        (status = 200, description = "Successfully deleted the user staff note"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User staff note not found"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<(i32, i64)>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let (user_id, staff_note_id) = path.into_inner();

    // like editing, removing a note is allowed to its author during the 24 hours following its
    // creation, without the permission
    let has_permission = arc
        .pool
        .user_has_permission(current_user.sub, &UserPermission::EditUserStaffNotes)
        .await?;

    arc.pool
        .check_user_staff_note_can_be_edited(
            user_id,
            staff_note_id,
            current_user.sub,
            has_permission,
        )
        .await?;

    arc.pool.delete_user_staff_note(staff_note_id).await?;

    Ok(HttpResponse::Ok().finish())
}
