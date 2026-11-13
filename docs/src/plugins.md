# Plugins

Plugins let an instance add custom behaviour without forking arcadia and without merging anything
upstream. A plugin is a **separate service** that arcadia calls over HTTP, so it can be written in
any language and deployed independently.

For now, plugins can add **external sources** (scrapers), like the built in TMDB, MusicBrainz, ISBN
and Comic Vine ones, and collect the **ratings** of the title groups on external websites, like the
built in TMDB ones.

## Configuration

Declare your plugins in the `scrapers` section of `config.yml`, at the root of the repository:

```yaml
scrapers:
  - id: anime
    placeholder: Anime url
    sources:
      AniDB:
        content_types:
          - tv_show
        domain: anidb.net
        icon: https://anidb.net/favicon.ico
      MyAnimeList:
        content_types:
          - tv_show
          - movie
        domain: myanimelist.net
        icon: https://myanimelist.net/favicon.ico
    title_url: http://anime-plugin:9000/scrape
    ratings_url: http://anime-plugin:9000/ratings
    timeout_seconds: 30
```

| Field | Description |
| --- | --- |
| `id` | Identifier used in the route `/api/external-sources/<id>`. Must not clash with a built in source (`tmdb`, `musicbrainz`, `isbn`, `comic-vine`). |
| `placeholder` | Placeholder of the input field displayed in the interface, also used as the source's display name. |
| `sources` | The websites the plugin accepts links from, by name, each with the `content_types` it supports (`movie`, `video`, `tv_show`, `music`, `podcast`, `software`, `book`, `live_performance`, `collection`) and the `domain` and `icon` of the website: the plugin is only asked for the ratings of the links on that domain (or one of its subdomains), displayed with the website's name and icon. A single plugin may serve several websites, dispatching on the link it is given. They are listed in a tooltip next to the input on the upload page once there are several of them, and the source is offered for every content type at least one of them supports. |
| `title_url` | Endpoint scraping a title, see [Writing a scraper plugin](#writing-a-scraper-plugin). |
| `ratings_url` | Optional, endpoint collecting ratings, see [Collecting ratings](#collecting-ratings). |
| `timeout_seconds` | Optional, defaults to `30`. |

`config.yml` is read once at startup. When the `scrapers` section is absent, no plugin is
registered. The icons are added to the frontend at build time, so it has to be rebuilt after changing
them.

Both `config.yml` and `compose.override.yml` are git ignored. `compose.yml` already mounts
`config.yml` into the backend container, so only the plugin services themselves have to be declared
in `compose.override.yml`, which Docker Compose loads automatically on top of `compose.yml`:

```yaml
services:
  anime-plugin:
    build: ../anime-plugin
    restart: unless-stopped
```

## Writing a scraper plugin

The `title_url` endpoint is called with the identifier the user typed as the `url` query parameter,
and the content type the uploader picked as `content_type` when they picked one:

```
GET http://anime-plugin:9000/scrape?url=https://anidb.net/anime/1234&content_type=tv_show
```

`content_type` is context rather than an order: it may be absent, and the plugin may answer with
another one.

It must answer with JSON:

```json
{
  "title_group": { "name": "...", "description": "...", "content_type": "tv_show", "...": "..." },
  "edition_group": null,
  "affiliated_artists": [
    {
      "name": "Some Person",
      "aliases": [],
      "description": "",
      "pictures": ["https://example.com/picture.jpg"],
      "external_links": ["https://example.com/person/1"],
      "roles": ["director"],
      "nickname": null
    }
  ]
}
```

`title_group` follows the `UserCreatedTitleGroup` schema and `edition_group` the
`UserCreatedEditionGroup` one; both are documented in the OpenAPI specification. All three fields are optional.

Artists are given **by name**: arcadia creates them, merges the roles of an artist appearing several
times, and returns real affiliated artists to
the interface. Arcadia also checks beforehand whether a title group already has the submitted link,
exactly as it was typed. It does not add that link to the scraped title group: the plugin puts the
source link in the title group's `external_links` itself, cleaned up as it sees fit.

`external_links` is optional and holds the artist's pages on public databases and other trackers.

Pictures are rehosted if image rehosting is enabled.

Anything else the plugin needs (API keys, caching, rate limiting) is its own business.

### Reporting a failure

A plugin that cannot scrape answers with a status outside the `2xx` range and a body holding the
message meant for the uploader:

```json
{ "error": "www.example.com answered with 503 Service Unavailable" }
```

Arcadia shows that message as is, and answers the interface with a `502` whatever status the plugin
used. A failure with no such body, an unreachable plugin, and an answer arcadia cannot read are
reported as a generic error instead, the details only being logged. Write the message for the
uploader: what they can act on (a wrong url, a page holding nothing, a site that is down), never a
stack trace or an internal identifier.

## Collecting ratings

When a title group is created or edited, the `ratings_url` endpoint of every plugin declaring one is
asked for the rating of each external link of the title group on the `domain` (or a subdomain) of
one of its websites, in the background. Then, every
`public_ratings_fetch_seconds` (in the `periodic_tasks` section), it is asked again about each link
at most once every `public_ratings_retry_seconds`, whatever it answered last time. The
`Fetch missing public ratings` maintenance tool asks about the links of the title groups created
before the plugin was configured.

```
GET http://anime-plugin:9000/ratings?url=https://anidb.net/anime/1234&last_collected_at=2026-10-01T12:00:00Z
```

`last_collected_at` is when the stored rating was collected, absent when none was yet. The plugin
decides on its own whether the rating needs a refresh, and answers with one of:

```json
{ "status": "rated", "rating": 8.2, "votes": 1234 }
{ "status": "retry_later" }
{ "status": "unsupported" }
```

- `rated`: the average rating, on the scale of the website, stored and displayed as is.
- `retry_later`: no new rating for now, e.g. the stored one is recent enough or the website is down.
  The stored rating, if any, is kept.
- `unsupported`: the plugin does not handle the link. It is no longer asked about it, until the title
  group is edited or the `Fetch missing public ratings` maintenance tool is run.

A failure (an unreachable plugin, a status outside the `2xx` range, an answer arcadia cannot read)
is only logged, and treated like `retry_later`.
