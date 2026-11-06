use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserClass, redis::RedisPoolInterface};

#[utoipa::path(
    get,
    operation_id = "Get all user classes",
    tag = "User Class",
    path = "/api/user-classes",
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Successfully retrieved user classes", body=Vec<UserClass>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    _user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let user_classes = arc.pool.get_all_user_classes().await?;

    Ok(HttpResponse::Ok().json(user_classes))
}
