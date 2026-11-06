use crate::Arcadia;
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::forum::{ForumOverview, ForumSearchQuery},
    redis::RedisPoolInterface,
};
use serde_json::json;

#[utoipa::path(
    get,
    operation_id = "Get forum",
    tag = "Forum",
    path = "/api/forum",
    responses(
        (status = 200, description = "Returns an overview of the forum", body=ForumOverview),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(arc: Data<Arcadia<R>>) -> Result<HttpResponse> {
    //TODO: restrict access to some sub_categories based on forbidden_classes
    let forum_categories = arc.pool.find_forum_cateogries_hierarchy().await?;
    let search_forum_threads_form = ForumSearchQuery {
        thread_name: None,
        page_size: 5,
        page: 1,
    };
    let latest_posts_in_threads = arc
        .pool
        .search_forum_threads(&search_forum_threads_form)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "forum_categories": forum_categories,
        "latest_posts_in_threads": latest_posts_in_threads.results
    })))
}
