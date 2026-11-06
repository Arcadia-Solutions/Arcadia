use crate::Arcadia;
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{models::user_badge::UserBadgeCategory, redis::RedisPoolInterface};

#[utoipa::path(
    get,
    operation_id = "List user badge categories",
    tag = "User Badge Category",
    path = "/api/user-badge-categories",
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "List of user badge categories", body=Vec<UserBadgeCategory>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(arc: Data<Arcadia<R>>) -> Result<HttpResponse> {
    let categories = arc.pool.find_all_user_badge_categories().await?;
    Ok(HttpResponse::Ok().json(categories))
}
