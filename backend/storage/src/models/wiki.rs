use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use crate::utils::compute_diff;

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct WikiArticle {
    pub id: i64,
    pub title: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by_id: i32,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    pub updated_by_id: i32,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedWikiArticle {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct EditedWikiArticle {
    pub id: i64,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SearchWikiQuery {
    pub search_string: String,
    pub title_only: bool,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct WikiSearchResult {
    pub id: i64,
    pub title: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SimilarWikiArticlesLink {
    pub wiki_article_id_1: i64,
    pub wiki_article_id_2: i64,
}

impl WikiArticle {
    pub fn diff(&self, edited: &EditedWikiArticle) -> Option<Value> {
        compute_diff(self, edited, &["id"])
    }
}
