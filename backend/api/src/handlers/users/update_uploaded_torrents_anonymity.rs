use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UpdateUploadedTorrentsAnonymity, redis::RedisPoolInterface};

#[utoipa::path(
    put,
    operation_id = "Update uploaded torrents anonymity",
    tag = "User",
    path = "/api/users/uploaded-torrents-anonymity",
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully updated the anonymity of the uploaded torrents"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<UpdateUploadedTorrentsAnonymity>,
    user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    arc.pool
        .update_all_uploaded_torrents_anonymity(user.sub, form.anonymous)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
