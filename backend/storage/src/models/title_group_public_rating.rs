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
    pub title_group_id: i32,
    pub external_link: String,
    pub provider: String,
    /// when the stored rating was collected, `None` if none has been yet
    pub collected_at: Option<DateTime<Utc>>,
    pub title_group_original_release_date: Option<NaiveDate>,
}

/// A rating answered by a provider.
#[derive(Debug, Deserialize)]
pub struct CollectedPublicRating {
    pub rating: f64,
    pub votes: i64,
}
