use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_storage::{models::title_group::TitleGroupLite, redis::RedisPoolInterface};

use crate::{Arcadia, handlers::title_groups::get_title_group::GetTitleGroupQuery};
use arcadia_common::error::Result;

pub type GetTitleGroupLiteQuery = GetTitleGroupQuery;

#[utoipa::path(
    get,
    operation_id = "Get title group info lite",
    tag = "Title Group",
    path = "/api/title-groups/lite",
    params(GetTitleGroupLiteQuery),
    responses(
        (status = 200, description = "Successfully got the title_group (lite info)", body=Vec<TitleGroupLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    arc: Data<Arcadia<R>>,
    query: Query<GetTitleGroupLiteQuery>,
) -> Result<HttpResponse> {
    let title_group = arc
        .pool
        .find_title_group_info_lite(Some(query.id), None, &None, 1)
        .await?;

    Ok(HttpResponse::Ok().json(title_group))
}
