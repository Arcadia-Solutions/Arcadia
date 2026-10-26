use crate::{
    handlers::scrapers::ExternalDBData,
    middlewares::auth_middleware::Authdata,
    services::external_db_service::{
        check_if_existing_title_group_with_link_exists, respond_with_scraped_data,
    },
    Arcadia,
};
use actix_web::{web::Data, HttpResponse};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{
        artist::{AffiliatedArtistHierarchy, UserCreatedAffiliatedArtist, UserCreatedArtist},
        edition_group::{create_default_edition_group, UserCreatedEditionGroup},
        title_group::{create_default_title_group, ContentType, UserCreatedTitleGroup},
    },
    redis::RedisPoolInterface,
};
use chrono::{DateTime, NaiveDate};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Description {
    Typed {
        #[expect(unused)]
        #[serde(rename = "type")]
        type_: String,
        value: String,
    },
    Untyped(String),
}

impl AsRef<str> for Description {
    fn as_ref(&self) -> &str {
        match self {
            Description::Typed { value, .. } => value,
            Description::Untyped(value) => value,
        }
    }
}

impl From<Description> for String {
    fn from(d: Description) -> Self {
        match d {
            Description::Typed { value, .. } => value,
            Description::Untyped(value) => value,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Work {
    title: String,
    description: Option<Description>,
    first_publish_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkLink {
    key: String,
}

#[derive(Debug, Deserialize, Clone)]
struct AuthorBio {
    value: String,
}

#[derive(Debug, Deserialize)]
struct Author {
    name: String,
    bio: Option<AuthorBio>,
}

#[derive(Debug, Deserialize)]
struct AuthorLink {
    key: String,
}

#[derive(Debug, Deserialize)]
struct Book {
    works: Vec<WorkLink>,
    authors: Vec<AuthorLink>,
    #[serde(rename = "covers")]
    cover_ids: Option<Vec<i64>>,
    isbn_13: Option<Vec<String>>,
}

fn parse_date(date: &str) -> Option<NaiveDate> {
    date.parse::<i32>()
        .ok()
        .and_then(|y| NaiveDate::from_ymd_opt(y, 1, 1))
        .or_else(|| NaiveDate::parse_from_str(date, "%B %d, %Y").ok())
}

/// `isbn` is the isbn of the book: every external source is called with the same query parameter,
/// which lets the interface call all of them the same way.
pub async fn exec<R: RedisPoolInterface + 'static>(
    isbn: &str,
    arc: &Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let book_url = format!("https://openlibrary.org/isbn/{isbn}.json");
    let book = reqwest::get(&book_url).await?.json::<Book>().await?;
    // there usually is only 1 work for an isbn (and there should only be 1 in theory as well)
    let work_path = &book.works.first().unwrap().key;
    let work_url = format!("https://openlibrary.org/{}.json", &work_path);
    let work = reqwest::get(&work_url).await?.json::<Work>().await?;

    let work_group_url = format!("https://openlibrary.org{}", work_path);

    if let Some(response) =
        check_if_existing_title_group_with_link_exists(&arc.pool, &work_group_url).await?
    {
        return Ok(response);
    }

    let original_release_date = work.first_publish_date.as_ref().and_then(|d| parse_date(d));

    let description = work
        .description
        .map(String::from)
        .unwrap_or_else(|| "".to_string());

    let mut authors: Vec<(Author, String)> = vec![];

    for link in book.authors {
        let author = reqwest::get(format!("https://openlibrary.org{}.json", link.key))
            .await?
            .json::<Author>()
            .await?;
        authors.push((author, format!("https://openlibrary.org{}", link.key)));
    }

    let artists = arc
        .pool
        .create_artists(
            &authors
                .iter()
                .map(|(author, author_link)| UserCreatedArtist {
                    name: author.name.clone(),
                    aliases: vec![],
                    description: author
                        .bio
                        .clone()
                        .unwrap_or(AuthorBio {
                            value: "".to_string(),
                        })
                        .value,
                    pictures: vec![],
                    external_links: vec![author_link.clone()],
                })
                .collect::<Vec<UserCreatedArtist>>(),
            user.sub,
        )
        .await?;

    let covers = if let Some(cover_id) = book.cover_ids.unwrap_or(vec![]).first() {
        vec![format!(
            "https://covers.openlibrary.org/b/id/{}-L.jpg",
            cover_id
        )]
    } else {
        vec![]
    };

    let title_group = UserCreatedTitleGroup {
        name: work.title,
        description,
        external_links: vec![work_group_url],
        original_release_date,
        covers,
        content_type: ContentType::Book,
        affiliated_artists: artists
            .iter()
            .map(|artist| UserCreatedAffiliatedArtist {
                artist_id: artist.id,
                title_group_id: 0,
                roles: vec![],
                nickname: None,
            })
            .collect(),
        ..create_default_title_group()
    };

    let edition_group = UserCreatedEditionGroup {
        additional_information: Some(
            json!({"isbn_13": book.isbn_13.unwrap_or(vec![]).first().unwrap_or(&"".to_string())}),
        ),
        ..create_default_edition_group()
    };

    let external_db_data = ExternalDBData {
        title_group: Some(title_group),
        edition_group: Some(edition_group),
        affiliated_artists: artists
            .iter()
            .map(|artist| AffiliatedArtistHierarchy {
                id: 0,
                artist: artist.clone(),
                title_group_id: 0,
                artist_id: artist.id,
                roles: vec![],
                nickname: None,
                created_at: DateTime::default(),
                created_by_id: 0,
            })
            .collect::<Vec<AffiliatedArtistHierarchy>>(),
        existing_title_group_id: None,
    };

    respond_with_scraped_data(&arc.pool, &arc.image_host, external_db_data).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_deserialization_moby_dick() {
        let moby_dick = serde_json::from_str::<Work>(include_str!("testdata/OL102749W.json"));

        assert!(moby_dick.is_ok());

        let moby_dick = moby_dick.unwrap();

        assert_eq!(moby_dick.title, "Moby Dick");
        assert!(moby_dick.description.is_some());
        assert_eq!(
            moby_dick.description.unwrap().as_ref(),
            "\"Command the murderous chalices! Drink ye harpooners! Drink and swear, ye men that man the deathful whaleboat's bow -- Death to Moby Dick!\" So Captain Ahab binds his crew to fulfil his obsession -- the destruction of the great white whale. Under his lordly but maniacal command the Pequod's commercial mission is perverted to one of vengeance. To Ahab, the monster that destroyed his body is not a creature, but the symbol of \"some unknown but still reasoning thing.\" Uncowed by natural disasters, ill omens, even death, Ahab urges his ship towards \"the undeliverable, nameless perils of the whale.\" Key letters from Melville to Nathaniel Hawthorne are printed at the end of this volume. - Back cover."
        );
        assert!(moby_dick.first_publish_date.is_none());
    }

    #[test]
    fn test_deserialization_les_mis() {
        let les_mis = serde_json::from_str::<Work>(include_str!("testdata/OL1063588W.json"));

        assert!(les_mis.is_ok());

        let les_mis = les_mis.unwrap();

        assert_eq!(les_mis.title, "Les Misérables");
        assert!(les_mis.description.is_some());
        assert_eq!(
            les_mis.description.unwrap().as_ref(),
            "In this story of the trials of the peasant Jean Valjean--a man unjustly imprisoned, baffled by destiny, and hounded by his nemesis, the magnificently realized, ambiguously malevolent police detective Javert--Hugo achieves the sort of rare imaginative resonance that allows a work of art to transcend its genre."
        );
        assert_eq!(les_mis.first_publish_date, Some("1863".into()));
    }

    #[test]
    fn test_parse_date() {
        // OpenLibrary published date is not normalized, try a couple varieties.

        let date1 = parse_date("1970").unwrap();
        assert_eq!((date1.year(), date1.month(), date1.day()), (1970, 1, 1));

        let date2 = parse_date("February 19, 1994").unwrap();
        assert_eq!((date2.year(), date2.month(), date2.day()), (1994, 2, 19));
    }
}
