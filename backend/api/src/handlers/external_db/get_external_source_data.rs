use crate::{
    handlers::{
        external_db::{
            get_comic_vine_data, get_isbn_data, get_musicbrainz_data, get_plugin_data,
            get_tmdb_data,
        },
        scrapers::ExternalDBData,
    },
    middlewares::auth_middleware::Authdata,
    Arcadia,
};
use actix_web::{
    web::{Data, Path, Query},
    HttpResponse,
};
use arcadia_common::error::Result;
use arcadia_storage::{models::title_group::ContentType, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetExternalSourceDataQuery {
    /// The url of the resource on the external source, or its identifier for the sources that have
    /// no url (the isbn of a book, for example).
    url: String,
    /// The content type the uploader picked before pasting the url.
    content_type: Option<ContentType>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetExternalSourceDataPath {
    /// The id of the external source, as listed by the upload information route.
    source_id: String,
}

/// Gets the data of a resource from an external source, be it one bundled with arcadia or one
/// provided by a plugin declared by the instance administrator.
#[utoipa::path(
    get,
    operation_id = "Get external source data",
    tag = "External Source",
    path = "/api/external-sources/{source_id}",
    params(GetExternalSourceDataPath, GetExternalSourceDataQuery),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully got the external source data", body=ExternalDBData),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<GetExternalSourceDataPath>,
    query: Query<GetExternalSourceDataQuery>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    // the ids handled here are the ones declared in `BUILT_IN_EXTERNAL_SOURCES`,
    // every other id is looked up among the plugins
    match path.source_id.as_str() {
        "isbn" => get_isbn_data::exec(&query.url, &arc, user).await,
        "tmdb" => get_tmdb_data::exec(&query.url, &arc, user).await,
        "comic-vine" => get_comic_vine_data::exec(&query.url, &arc).await,
        "musicbrainz" => get_musicbrainz_data::exec(&query.url, &arc).await,
        source_id => {
            get_plugin_data::exec(
                source_id,
                &query.url,
                query.content_type.clone(),
                &arc,
                user,
            )
            .await
        }
    }
}
