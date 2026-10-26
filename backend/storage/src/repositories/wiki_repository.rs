use crate::{
    connection_pool::ConnectionPool,
    models::{
        common::PaginatedResults,
        wiki::{
            EditedWikiArticle, SearchWikiQuery, SimilarWikiArticlesLink, UserCreatedWikiArticle,
            WikiArticle, WikiSearchResult,
        },
    },
};
use arcadia_common::error::{Error, Result};
use serde_json::Value;

impl ConnectionPool {
    pub async fn create_wiki_article(
        &self,
        article: &UserCreatedWikiArticle,
        current_user_id: i32,
    ) -> Result<WikiArticle> {
        let created_article = sqlx::query_as!(
            WikiArticle,
            r#"
                INSERT INTO wiki_articles (title, body, created_by_id, updated_by_id)
                VALUES ($1, $2, $3, $4)
                RETURNING id, title, created_at, created_by_id, updated_at, updated_by_id, body
            "#,
            article.title,
            article.body,
            current_user_id,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateWikiArticle)?;

        Ok(created_article)
    }

    pub async fn find_wiki_article_raw(&self, article_id: i64) -> Result<WikiArticle> {
        sqlx::query_as!(
            WikiArticle,
            r#"SELECT id, title, created_at, created_by_id, updated_at, updated_by_id, body FROM wiki_articles WHERE id = $1"#,
            article_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindWikiArticle)
    }

    pub async fn find_wiki_article(&self, article_id: i64) -> Result<Value> {
        let article = sqlx::query!(
            r#"
            SELECT
                json_build_object(
                    'id', wa.id,
                    'title', wa.title,
                    'created_at', wa.created_at,
                    'created_by', json_build_object(
                        'id', cb.id,
                        'username', cb.username,
                        'warned', cb.warned,
                        'banned', cb.banned
                    ),
                    'updated_at', wa.updated_at,
                    'updated_by', json_build_object(
                        'id', ub.id,
                        'username', ub.username,
                        'warned', ub.warned,
                        'banned', ub.banned
                    ),
                    'body', wa.body,
                    'similar_wiki_articles', COALESCE((
                        SELECT json_agg(json_build_object('id', other.id, 'title', other.title) ORDER BY other.title)
                        FROM similar_wiki_articles swa
                        JOIN wiki_articles other ON other.id = CASE
                            WHEN swa.wiki_article_id_1 = wa.id THEN swa.wiki_article_id_2
                            ELSE swa.wiki_article_id_1
                        END
                        WHERE swa.wiki_article_id_1 = wa.id OR swa.wiki_article_id_2 = wa.id
                    ), '[]'::json)
                ) AS article_json
            FROM
                wiki_articles wa
            JOIN
                users cb ON wa.created_by_id = cb.id
            JOIN
                users ub ON wa.updated_by_id = ub.id
            WHERE
                wa.id = $1
            "#,
            article_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotFindWikiArticle)?;

        Ok(article.article_json.unwrap())
    }

    pub async fn link_similar_wiki_articles(
        &self,
        link: &SimilarWikiArticlesLink,
        current_user_id: i32,
    ) -> Result<()> {
        if link.wiki_article_id_1 == link.wiki_article_id_2 {
            return Err(Error::WikiArticleCannotBeLinkedToItself);
        }

        let (id_1, id_2) = if link.wiki_article_id_1 < link.wiki_article_id_2 {
            (link.wiki_article_id_1, link.wiki_article_id_2)
        } else {
            (link.wiki_article_id_2, link.wiki_article_id_1)
        };

        sqlx::query!(
            r#"
                INSERT INTO similar_wiki_articles (wiki_article_id_1, wiki_article_id_2, created_by_id)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
            "#,
            id_1,
            id_2,
            current_user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotLinkSimilarWikiArticles)?;

        Ok(())
    }

    pub async fn unlink_similar_wiki_articles(&self, link: &SimilarWikiArticlesLink) -> Result<()> {
        if link.wiki_article_id_1 == link.wiki_article_id_2 {
            return Err(Error::WikiArticleCannotBeLinkedToItself);
        }

        let (id_1, id_2) = if link.wiki_article_id_1 < link.wiki_article_id_2 {
            (link.wiki_article_id_1, link.wiki_article_id_2)
        } else {
            (link.wiki_article_id_2, link.wiki_article_id_1)
        };

        sqlx::query!(
            r#"
                DELETE FROM similar_wiki_articles
                WHERE wiki_article_id_1 = $1 AND wiki_article_id_2 = $2
            "#,
            id_1,
            id_2,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotUnlinkSimilarWikiArticles)?;

        Ok(())
    }

    pub async fn edit_wiki_article(
        &self,
        article: &EditedWikiArticle,
        current_user_id: i32,
    ) -> Result<WikiArticle> {
        let created_article = sqlx::query_as!(
            WikiArticle,
            r#"
                UPDATE wiki_articles
                SET title = $1, body = $2, updated_by_id = $3, updated_at = NOW()
                WHERE id = $4
                RETURNING id, title, created_at, created_by_id, updated_at, updated_by_id, body
            "#,
            article.title,
            article.body,
            current_user_id,
            article.id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateWikiArticle)?;

        Ok(created_article)
    }

    pub async fn search_wiki_articles(
        &self,
        form: &SearchWikiQuery,
    ) -> Result<PaginatedResults<WikiSearchResult>> {
        let offset = (form.page - 1) * form.page_size;

        let total_items: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM wiki_articles
            WHERE unaccent(title) ILIKE '%' || unaccent($1) || '%'
               OR ($2 = false AND unaccent(body) ILIKE '%' || unaccent($1) || '%')
            "#,
            form.search_string,
            form.title_only,
        )
        .fetch_one(&self.pool)
        .await
        .unwrap()
        .unwrap();

        let results = sqlx::query_as!(
            WikiSearchResult,
            r#"
            SELECT id, title, created_at, updated_at
            FROM wiki_articles
            WHERE unaccent(title) ILIKE '%' || unaccent($1) || '%'
               OR ($2 = false AND unaccent(body) ILIKE '%' || unaccent($1) || '%')
            ORDER BY created_at DESC
            OFFSET $3 LIMIT $4
            "#,
            form.search_string,
            form.title_only,
            offset as i64,
            form.page_size as i64,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResults {
            results,
            total_items,
            page: form.page,
            page_size: form.page_size,
        })
    }
}
