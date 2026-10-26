use chrono::{DateTime, NaiveDate, Utc};
use musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{prelude::FromRow, types::Json};
use utoipa::ToSchema;

use super::{
    artist::{AffiliatedArtistHierarchy, AffiliatedArtistLite, UserCreatedAffiliatedArtist},
    edition_group::{EditionGroupHierarchy, EditionGroupHierarchyLite, EditionGroupInfoLite},
    series::SeriesLite,
    title_group_comment::TitleGroupCommentHierarchy,
};
use crate::models::{
    collage::CollageSearchResult, entity::AffiliatedEntityHierarchy, torrent::Language,
    torrent_request::TorrentRequestHierarchyLite, user::UserLite,
};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema, Clone, PartialEq, Eq)]
#[sqlx(type_name = "content_type_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Movie,
    Video,
    #[sqlx(rename = "tv_show")]
    #[serde(rename = "tv_show")]
    TVShow,
    Music,
    Podcast,
    Software,
    Book,
    LivePerformance,
    // aka SiteRip, but also includes packs of other content than website dumps (books, etc.)
    // this allows users to group content (when possible), which lowers the load on the tracker and makes the upload process faster (1 announce instead of multiple)
    Collection,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema, Clone)]
#[sqlx(type_name = "platform_enum")]
pub enum Platform {
    Windows,
    DOS,
    Linux,
    MacOS,
    IOS,
    AppleBandaiPippin,
    AppleII,
    Android,
    Xbox,
    Xbox360,
    XboxOne,
    XboxSeriesXS,
    Playstation,
    Playstation2,
    Playstation3,
    Playstation4,
    Playstation5,
    PlaystationPortable,
    PlaystationVita,
    GameBoy,
    GameBoyAdvance,
    GameBoyColor,
    NES,
    SNES,
    N64,
    ThreeDS,
    New3DS,
    DS,
    VirtualBoy,
    GameCube,
    Wii,
    WiiU,
    Switch,
    Switch2,
    Dreamcast,
    GameGear,
    MasterSystem,
    MegaDrive,
    Pico,
    Saturn,
    SG1000,
    Atari2600,
    Atari5200,
    Atari7800,
    AtariJaguar,
    AtariLynx,
    AtariST,
    AmstradCPC,
    BandaiWonderSwan,
    BandaiWonderSwanColor,
    Commodore64,
    Commodore128,
    CommodoreAmiga,
    AmigaCD32,
    CommodorePlus4,
    CommodoreVIC20,
}

// this is not to store the genre, but the format
#[derive(Debug, Serialize, Deserialize, sqlx::Type, ToSchema, Clone)]
#[sqlx(type_name = "title_group_category_enum")]
pub enum TitleGroupCategory {
    //music
    Ep,
    Album, // includes "live album" as an "edition"
    Single,
    Soundtrack,
    Anthology,
    Compilation,
    Remix,
    Bootleg,
    Mixtape,
    ConcertRecording,
    DjMix,
    // movies
    FeatureFilm,
    ShortFilm,
    // tv shows
    // none ?
    // software
    Game,
    Program,
    // written documents (books and their derivatives)
    Illustrated, // includes mangas, comics, visual novels
    Periodical,  //includes newspapers, magazines
    Book,        // hardcover, etc
    Article,     // includes studies, theseis, essais, etc.
    Manual,      // includes guides, music sheets, etc. for physical and digital products
    // should be used as little as possible
    Other,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::Type)]
pub enum ExternalDB {
    #[sqlx(rename = "tmdb")]
    #[serde(rename = "tmdb")]
    Tmdb,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PublicRating {
    pub service: ExternalDB,
    pub rating: f64,
    pub votes: i64,
}

// Every attribute is specific to the title,
// no specific information should be entered about the editions or the torrents
#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TitleGroup {
    pub id: i32,
    pub master_group_id: Option<i32>, // only if master groups are needed for this type of content
    pub name: String,
    pub name_aliases: Vec<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    pub created_by_id: i32,
    pub description: String,
    pub platform: Option<Platform>,
    pub original_language: Option<Language>,
    #[schema(value_type = String, format = Date, nullable = true)]
    pub original_release_date: Option<NaiveDate>,
    pub original_release_date_only_year_known: bool,
    pub tagline: Option<String>, // catchy sentence that represents the general idea of the title
    pub country_from: Option<String>,
    pub covers: Vec<String>,
    pub external_links: Vec<String>, // (public DBs, other trackers)
    pub trailers: Vec<String>,
    pub category: Option<TitleGroupCategory>, // ((movie: feature film, short film), (music: ep, album, compilation))
    pub content_type: ContentType,            // movies, tv shows, books, games, etc
    pub tags: Vec<String>,
    #[schema(value_type = Vec<PublicRating>)]
    pub public_ratings: Value, // {service: rating}
    pub series_id: Option<i64>,
    pub screenshots: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SimilarTitleGroupsLink {
    pub group_1: i32,
    pub group_2: i32,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SimilarTitleGroupLite {
    pub id: i32,
    pub name: String,
    #[schema(value_type = String, format = Date, nullable = true)]
    pub original_release_date: Option<NaiveDate>,
    pub cover: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserCreatedTitleGroup {
    pub name: String,
    pub name_aliases: Vec<String>,
    pub description: String,
    pub original_language: Option<Language>,
    pub country_from: Option<String>,
    pub covers: Vec<String>,
    pub external_links: Vec<String>,
    pub trailers: Vec<String>,
    // pub artists_affiliated: //(multiple categories, multiple in each category) (composer, remixer, actors, developers, etc.)
    // pub entities_affiliated (multiple categories, mutliple in each category) (publisher, record label, franchise, etc.)
    pub category: Option<TitleGroupCategory>, // ((movie: feature film, short film), (music: ep, album, compilation))
    pub content_type: ContentType,            // movies, tv shows, books, games, etc
    pub tags: Vec<String>,
    pub tagline: Option<String>,
    pub platform: Option<Platform>,
    #[schema(value_type = String, format = Date, nullable = true)]
    pub original_release_date: Option<NaiveDate>,
    pub original_release_date_only_year_known: bool,
    pub affiliated_artists: Vec<UserCreatedAffiliatedArtist>,
    pub series_id: Option<i64>,
    pub screenshots: Vec<String>,
    // one of them should be given, if master groups are required for this type of content
    pub master_group_id: Option<i32>,
    // pub master_group: Option<UserCreatedMasterGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TitleGroupHierarchyLite {
    pub id: i32,
    pub name: String,
    pub covers: Vec<String>,
    pub category: Option<TitleGroupCategory>,
    pub content_type: ContentType,
    pub tags: Vec<String>,
    #[schema(value_type = String, format = Date, nullable = true)]
    pub original_release_date: Option<NaiveDate>,
    pub original_release_date_only_year_known: bool,
    pub platform: Option<Platform>,
    #[schema(value_type = Vec<EditionGroupHierarchyLite>)]
    pub edition_groups: Json<Vec<EditionGroupHierarchyLite>>,
    #[schema(value_type = Vec<AffiliatedArtistLite>)]
    pub affiliated_artists: Json<Vec<AffiliatedArtistLite>>,
    #[schema(value_type = Option<SeriesLite>)]
    pub series: Option<Json<SeriesLite>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TitleGroupLite {
    pub id: i32,
    pub name: String,
    pub content_type: ContentType,
    #[schema(value_type = String, format = Date, nullable = true)]
    pub original_release_date: Option<NaiveDate>,
    pub original_release_date_only_year_known: bool,
    pub covers: Vec<String>,
    pub edition_groups: Vec<EditionGroupInfoLite>,
    pub platform: Option<Platform>,
    pub series: Option<SeriesLite>,
    pub latest_torrent_uploaded_by: Option<UserLite>,
    #[schema(value_type = String, format = DateTime, nullable = true, required = false)]
    pub latest_torrent_uploaded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, sqlx::Decode)]
pub struct TitleGroupInfoLite {
    pub id: i32,
    pub name: String,
    pub content_type: ContentType,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MasterGroupEntry {
    pub id: i32,
    pub name: String,
    pub content_type: ContentType,
    pub platform: Option<Platform>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TitleGroupAndAssociatedData {
    pub title_group: TitleGroup,
    pub edition_groups: Vec<EditionGroupHierarchy>,
    pub series: Option<SeriesLite>,
    pub affiliated_artists: Vec<AffiliatedArtistHierarchy>,
    pub affiliated_entities: Vec<AffiliatedEntityHierarchy>,
    pub title_group_comments: Vec<TitleGroupCommentHierarchy>,
    pub torrent_requests: Vec<TorrentRequestHierarchyLite>,
    pub is_subscribed_to_torrents: bool,
    pub is_subscribed_to_comments: bool,
    pub bookmark_id: Option<i64>,
    pub in_same_master_group: Vec<MasterGroupEntry>,
    pub collages: Vec<CollageSearchResult>,
    pub related_threads: Vec<crate::models::forum::RelatedForumThread>,
    pub similar_title_groups: Vec<SimilarTitleGroupLite>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct EditedTitleGroup {
    pub id: i32,
    pub master_group_id: Option<i32>,
    pub name: String,
    pub name_aliases: Vec<String>,
    pub description: String,
    pub platform: Option<Platform>,
    pub original_language: Option<Language>,
    #[schema(value_type = String, format = Date, nullable = true)]
    pub original_release_date: Option<NaiveDate>,
    pub original_release_date_only_year_known: bool,
    pub tagline: Option<String>,
    pub country_from: Option<String>,
    pub covers: Vec<String>,
    pub external_links: Vec<String>,
    pub trailers: Vec<String>,
    pub category: Option<TitleGroupCategory>,
    pub content_type: ContentType,
    pub screenshots: Vec<String>,
}

pub fn create_default_title_group() -> UserCreatedTitleGroup {
    UserCreatedTitleGroup {
        name: String::from(""),
        name_aliases: Vec::new(),
        description: String::from(""),
        original_language: None,
        country_from: None,
        covers: vec!["".to_string()],
        external_links: vec!["".to_string()],
        trailers: vec!["".to_string()],
        category: None,
        content_type: ContentType::Book,
        tags: Vec::new(),
        tagline: None,
        platform: None,
        original_release_date: Some(Utc::now().date_naive()),
        original_release_date_only_year_known: false,
        affiliated_artists: Vec::new(),
        series_id: None,
        screenshots: Vec::new(),
        master_group_id: None,
    }
}

impl From<ReleaseGroupPrimaryType> for TitleGroupCategory {
    fn from(rg_type: ReleaseGroupPrimaryType) -> Self {
        match rg_type {
            ReleaseGroupPrimaryType::Album => TitleGroupCategory::Album,
            ReleaseGroupPrimaryType::Single => TitleGroupCategory::Single,
            ReleaseGroupPrimaryType::Ep => TitleGroupCategory::Ep,
            _ => TitleGroupCategory::Other,
        }
    }
}
