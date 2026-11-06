use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        user::UserPermission,
        user_staff_note::{UserCreatedStaffNote, UserStaffNoteWithAuthor},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create user staff note",
    tag = "User",
    path = "/api/users/{id}/staff-notes",
    security(("http" = ["Bearer"])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    request_body = UserCreatedStaffNote,
    responses(
        (status = 201, description = "Successfully created the user staff note", body=UserStaffNoteWithAuthor),
        (status = 403, description = "Insufficient permissions"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    user_id: Path<i32>,
    form: Json<UserCreatedStaffNote>,
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

    let staff_note = arc
        .pool
        .create_user_staff_note(*user_id, current_user.sub, &form.content)
        .await?;

    Ok(HttpResponse::Created().json(staff_note))
}
