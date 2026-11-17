use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

use arcadia_storage::models::{
    artist::{AffiliatedArtistHierarchy, ArtistRole},
    edition_group::UserCreatedEditionGroup,
    title_group::{ContentType, UserCreatedTitleGroup},
};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExternalDBData {
    pub title_group: Option<UserCreatedTitleGroup>,
    pub edition_group: Option<UserCreatedEditionGroup>,
    pub affiliated_artists: Vec<AffiliatedArtistHierarchy>, // pub series: UserCreatedSeries
    pub existing_title_group_id: Option<i32>,
}

/// An artist as returned by a scraper, before it has been created in the database.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ScrapedAffiliatedArtist {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub pictures: Vec<String>,
    #[serde(default)]
    pub external_links: Vec<String>,
    pub roles: Vec<ArtistRole>,
    pub nickname: Option<String>,
}

/// The data returned by an external source plugin. Artists are given by name,
/// arcadia creates them and turns them into affiliated artists.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ScrapedExternalData {
    pub title_group: Option<UserCreatedTitleGroup>,
    pub edition_group: Option<UserCreatedEditionGroup>,
    #[serde(default)]
    pub affiliated_artists: Vec<ScrapedAffiliatedArtist>,
    /// The rating found on the scraped page, saved before the title group is created so it does
    /// not have to be asked for again.
    pub public_rating: Option<ScrapedPublicRating>,
}

/// A rating returned by an external source plugin along with the title.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ScrapedPublicRating {
    /// one of the title group's `external_links`, as cleaned by the plugin
    pub external_link: String,
    pub rating: f64,
    pub votes: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ExternalSource {
    pub id: String,
    pub placeholder: String,
    /// The websites the source accepts links from, each with the content types it supports.
    /// The interface shows the input for any content type supported by at least one of them,
    /// and lists them next to it once the source holds more than one.
    pub sources: BTreeMap<String, Vec<ContentType>>,
}
