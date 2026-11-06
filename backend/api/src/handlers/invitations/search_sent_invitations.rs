use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        common::PaginatedResults,
        invitation::{InvitationHierarchy, SearchSentInvitationsQuery},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    get,
    operation_id = "Search sent invitations",
    tag = "Invitation",
    path = "/api/invitations",
    params(SearchSentInvitationsQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the sent invitations", body = PaginatedResults<InvitationHierarchy>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<SearchSentInvitationsQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    if query.show_foreign_invitations {
        arc.pool
            .require_permission(
                user.sub,
                &UserPermission::ViewForeignInvitations,
                req.path(),
            )
            .await?;
    }

    let invitations = arc
        .pool
        .search_sent_invitations(&query, user.sub, query.show_foreign_invitations)
        .await?;

    Ok(HttpResponse::Ok().json(invitations))
}
