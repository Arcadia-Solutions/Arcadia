use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{donation::DeletedDonation, user::UserPermission},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    delete,
    operation_id = "Delete donation",
    tag = "Donation",
    path = "/api/donations",
    security(
        ("http" = ["Bearer"])
    ),
    request_body = DeletedDonation,
    responses(
        (status = 200, description = "Donation deleted successfully"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: Json<DeletedDonation>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::DeleteDonation, req.path())
        .await?;

    arc.pool.find_donation_by_id(request.id).await?;

    arc.pool.delete_donation(request.id).await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
