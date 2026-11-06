pub mod get_comic_vine_data;
pub mod get_external_source_data;
pub mod get_isbn_data;
pub mod get_musicbrainz_data;
pub mod get_plugin_data;
pub mod get_tmdb_data;

use actix_web::web::{ServiceConfig, get, resource};
use arcadia_storage::{models::title_group::ContentType, redis::RedisPoolInterface};
use std::collections::BTreeMap;

use crate::handlers::scrapers::ExternalSource;

/// The external sources bundled with arcadia, as `(id, placeholder, website, content_types)`.
/// Each id is dispatched to its handler in `get_external_source_data`, and each one accepts
/// links from a single website.
const BUILT_IN_EXTERNAL_SOURCES: &[(&str, &str, &str, &[ContentType])] = &[
    ("isbn", "ISBN", "ISBN", &[ContentType::Book]),
    (
        "tmdb",
        "TMDB url",
        "TMDB",
        &[ContentType::Movie, ContentType::TVShow],
    ),
    (
        "comic-vine",
        "Comic Vine url",
        "Comic Vine",
        &[ContentType::Book],
    ),
    (
        "musicbrainz",
        "MusicBrainz url",
        "MusicBrainz",
        &[ContentType::Music],
    ),
];

pub fn built_in_external_sources() -> Vec<ExternalSource> {
    BUILT_IN_EXTERNAL_SOURCES
        .iter()
        .map(|(id, placeholder, website, content_types)| ExternalSource {
            id: id.to_string(),
            placeholder: placeholder.to_string(),
            sources: BTreeMap::from([(website.to_string(), content_types.to_vec())]),
        })
        .collect()
}

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    // a single route serves every external source, built in ones as well as plugins
    cfg.service(
        resource("/{source_id}").route(get().to(self::get_external_source_data::exec::<R>)),
    );
}
