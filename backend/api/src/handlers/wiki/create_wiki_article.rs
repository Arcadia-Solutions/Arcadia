use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        user::UserPermission,
        wiki::{UserCreatedWikiArticle, WikiArticle},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create wiki article",
    tag = "Wiki",
    path = "/api/wiki/articles",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully created the wiki article", body=WikiArticle),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    article: Json<UserCreatedWikiArticle>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::CreateWikiArticle, req.path())
        .await?;

    let article = arc.pool.create_wiki_article(&article, user.sub).await?;

    Ok(HttpResponse::Created().json(article))
}
