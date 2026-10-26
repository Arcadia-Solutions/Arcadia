use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::utils::compute_diff;
use arcadia_storage::{
    models::{
        user::UserPermission,
        user_edit_change_log::NewUserEditChangeLog,
        wiki::{EditedWikiArticle, WikiArticle},
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit wiki article",
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
    article: Json<EditedWikiArticle>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditWikiArticle, req.path())
        .await?;

    let original_article = arc.pool.find_wiki_article_raw(article.id).await?;

    if let Some(edits) = compute_diff(&original_article, &article, &["id"]) {
        arc.pool
            .create_user_edit_change_log(&NewUserEditChangeLog {
                item_type: "wiki_article".to_string(),
                item_id: original_article.id,
                edited_by_id: user.sub,
                edits,
            })
            .await?;
    }

    let article = arc.pool.edit_wiki_article(&article, user.sub).await?;

    Ok(HttpResponse::Created().json(article))
}
