use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserPermission, redis::RedisPoolInterface};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetTorrentStaffChecked {
    pub torrent_id: i32,
    pub staff_checked: bool,
}

#[utoipa::path(
    put,
    operation_id = "Set torrent staff checked",
    tag = "Torrent",
    path = "/api/torrents/staff-checked",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Torrent staff_checked status updated"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: Json<SetTorrentStaffChecked>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(
            user.sub,
            &UserPermission::SetTorrentStaffChecked,
            req.path(),
        )
        .await?;

    arc.pool
        .set_torrent_staff_checked(form.torrent_id, form.staff_checked)
        .await?;

    Ok(HttpResponse::Ok().json(json!({"result": "success"})))
}
