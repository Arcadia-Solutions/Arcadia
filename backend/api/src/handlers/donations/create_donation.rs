use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        donation::{Donation, UserCreatedDonation},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create donation",
    tag = "Donation",
    path = "/api/donations",
    security(
        ("http" = ["Bearer"])
    ),
    request_body = UserCreatedDonation,
    responses(
        (status = 201, description = "Successfully created the donation", body=Donation),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: Json<UserCreatedDonation>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::CreateDonation, req.path())
        .await?;

    arc.pool.find_user_with_id(request.donated_by_id).await?;

    if request.amount <= 0.0 {
        return Err(Error::DonationAmountMustBePositive);
    }

    let donation = arc.pool.create_donation(&request, user.sub).await?;

    Ok(HttpResponse::Created().json(donation))
}
