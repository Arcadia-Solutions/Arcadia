use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::FromRow;
use std::collections::HashMap;
use strum::Display;
use utoipa::{IntoParams, ToSchema};

use super::common::OrderByDirection;
use crate::utils::compute_diff;

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema, Clone)]
pub struct Artist {
    pub id: i64,
    pub name: String,
    pub aliases: Vec<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by_id: i32,
    pub description: String,
    pub pictures: Vec<String>,
    pub external_links: Vec<String>,
    pub title_groups_amount: i32,
    pub edition_groups_amount: i32,
    pub torrents_amount: i32,
    pub seeders_amount: i32,
    pub leechers_amount: i32,
    pub snatches_amount: i32,
    pub total_size: i64,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct UserCreatedArtist {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub pictures: Vec<String>,
    pub external_links: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct EditedArtist {
    pub id: i64,
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub pictures: Vec<String>,
    pub external_links: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, FromRow, ToSchema)]
pub struct ArtistLite {
    pub id: i64,
    pub name: String,
    pub aliases: Vec<String>,
    pub pictures: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Display)]
pub enum ArtistSearchOrderByColumn {
    #[serde(rename = "name")]
    #[strum(serialize = "name")]
    Name,
    #[serde(rename = "created_at")]
    #[strum(serialize = "created_at")]
    CreatedAt,
    #[serde(rename = "title_groups_amount")]
    #[strum(serialize = "title_groups_amount")]
    TitleGroupsAmount,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct SearchArtistsQuery {
    pub name: Option<String>,
    pub page: u32,
    pub page_size: u32,
    pub order_by_column: ArtistSearchOrderByColumn,
    pub order_by_direction: OrderByDirection,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ArtistSearchResult {
    pub id: i64,
    pub name: String,
    pub aliases: Vec<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by_id: i32,
    pub pictures: Vec<String>,
    pub title_groups_amount: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema, PartialEq)]
#[sqlx(type_name = "artist_role_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ArtistRole {
    Main,
    Guest,
    Producer,
    Director,
    Cinematographer,
    Actor,
    Writer,
    Composer,
    Remixer,
    Conductor,
    DjCompiler,
    Arranger,
    Host,
    Author,
    Illustrator,
    Editor,
    Developer,
    Designer,
    Creator,
    Performer,
    Presenter,
    Contributor,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AffiliatedArtist {
    pub id: i64,
    pub title_group_id: i32,
    pub artist_id: i64,
    pub roles: Vec<ArtistRole>,
    pub nickname: Option<String>, // for example: name of the character the actor is playing
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AffiliatedArtistLite {
    pub artist_id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedAffiliatedArtist {
    pub title_group_id: i32,
    pub artist_id: i64,
    pub roles: Vec<ArtistRole>,
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AffiliatedArtistHierarchy {
    pub id: i64,
    pub title_group_id: i32,
    pub artist_id: i64,
    pub roles: Vec<ArtistRole>,
    pub nickname: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub created_by_id: i32,
    pub artist: Artist,
}

impl Artist {
    pub fn diff(&self, edited: &EditedArtist) -> Option<Value> {
        compute_diff(self, edited, &["id"])
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ArtistEnriched {
    pub artist: Artist,
    pub is_subscribed_to_title_groups: bool,
    pub tags: HashMap<String, i64>,
    pub related_threads: Vec<crate::models::forum::RelatedForumThread>,
    pub title_group_stats: crate::models::title_group_stats::TitleGroupStatsResponse,
}
