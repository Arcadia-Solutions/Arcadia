use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_storage::{
    models::title_group::TitleGroupAndAssociatedData, redis::RedisPoolInterface,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use arcadia_common::error::Result;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetTitleGroupQuery {
    pub id: i32,
}

#[utoipa::path(
    get,
    operation_id = "Get title group",
    tag = "Title Group",
    path = "/api/title-groups",
    params(GetTitleGroupQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the title_group", body=TitleGroupAndAssociatedData),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<GetTitleGroupQuery>,
    user: Authdata,
) -> Result<HttpResponse> {
    let title_group_data = arc
        .pool
        .find_title_group_hierarchy(query.id, user.sub)
        .await?;

    // Mark any notification tied to this title group as read for this user
    let _ = arc
        .pool
        .mark_notification_title_group_comment_as_read(query.id, user.sub)
        .await;
    let _ = arc
        .pool
        .mark_notification_title_group_torrent_as_read(query.id, user.sub)
        .await;
    let _ = arc
        .pool
        .mark_notification_artist_title_group_as_read(query.id, user.sub)
        .await;

    Ok(HttpResponse::Ok().json(title_group_data))
}
