pub mod create_wiki_article;
pub mod edit_wiki_article;
pub mod get_wiki_article;
pub mod link_similar_wiki_articles;
pub mod unlink_similar_wiki_articles;

use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_storage::redis::RedisPoolInterface;

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("/articles")
            .route(post().to(self::create_wiki_article::exec::<R>))
            .route(put().to(self::edit_wiki_article::exec::<R>))
            .route(get().to(self::get_wiki_article::exec::<R>)),
    )
    .service(
        resource("/articles/similar")
            .route(post().to(self::link_similar_wiki_articles::exec::<R>))
            .route(delete().to(self::unlink_similar_wiki_articles::exec::<R>)),
    );
}
