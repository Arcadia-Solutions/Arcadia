use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A title group's rating on an external website, on the scale of that website.
#[derive(Debug, Serialize, ToSchema)]
pub struct PublicRating {
    pub external_link: String,
    /// the average rating, on the scale of the website
    pub rating: f64,
    pub votes: i64,
    #[schema(value_type = String, format = DateTime)]
    pub collected_at: DateTime<Utc>,
}

/// A rating a provider (TMDB or a scraper plugin) has to be asked for.
#[derive(Debug)]
pub struct PublicRatingToFetch {
    pub external_link: String,
    pub provider: String,
    /// when the stored rating was collected, `None` if none has been yet
    pub collected_at: Option<DateTime<Utc>>,
    /// when the provider was last asked about the link before this time, `None` if it never was
    pub last_tried_fetching_at: Option<DateTime<Utc>>,
    /// the latest original release date of the title groups listing the link
    pub original_release_date: Option<NaiveDate>,
}

/// A rating answered by a provider.
#[derive(Debug, Deserialize)]
pub struct CollectedPublicRating {
    pub rating: f64,
    pub votes: i64,
}
