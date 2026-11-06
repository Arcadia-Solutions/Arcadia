use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        user::{SearchUsersQuery, UserPermission, UserSearchResult},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Search users",
    tag = "Search",
    path = "/api/search/users",
    request_body = SearchUsersQuery,
    security(("http" = ["Bearer"])),
    description = "Search registered users with pagination. Case insensitive username search. \
                   Filtering on permissions requires the set_and_view_user_permissions permission.",
    responses(
        (status = 200, description = "Successfully searched users", body=PaginatedResults<UserSearchResult>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    body: Json<SearchUsersQuery>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let query = body.into_inner();

    arc.pool
        .require_permission(current_user.sub, &UserPermission::SearchUsers, req.path())
        .await?;

    // Filtering on permissions reveals which permissions a user has, so it is only allowed for
    // staff members that may also set and view them.
    if query
        .permissions
        .as_ref()
        .is_some_and(|permissions| !permissions.is_empty())
    {
        arc.pool
            .require_permission(
                current_user.sub,
                &UserPermission::SetAndViewUserPermissions,
                req.path(),
            )
            .await?;
    }

    let mut results = arc.pool.search_users(&query).await?;

    let can_see_paranoia_hidden_user_info = arc
        .pool
        .user_has_permission(current_user.sub, &UserPermission::SeeParanoiaHiddenUserInfo)
        .await?;
    if !can_see_paranoia_hidden_user_info {
        for user in &mut results.results {
            if user.id != current_user.sub {
                user.hide_paranoia_hidden_stats();
            }
        }
    }

    Ok(HttpResponse::Ok().json(results))
}
