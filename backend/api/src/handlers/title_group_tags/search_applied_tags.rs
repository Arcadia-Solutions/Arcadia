use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Query},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        title_group_tag::{SearchTitleGroupAppliedTagsQuery, TitleGroupAppliedTag},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search title group applied tags",
    tag = "Title Group Tag",
    path = "/api/title-group-tags/applied",
    params(SearchTitleGroupAppliedTagsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Paginated list of the tags applied to title groups", body=PaginatedResults<TitleGroupAppliedTag>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchTitleGroupAppliedTagsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::ManageTitleGroupTags, req.path())
        .await?;

    let results = arc.pool.search_title_group_applied_tags(&query).await?;

    Ok(HttpResponse::Ok().json(results))
}
