# Plugins

Plugins let an instance add custom behaviour without forking arcadia and without merging anything
upstream. A plugin is a **separate service** that arcadia calls over HTTP, so it can be written in
any language and deployed independently.

For now, plugins can add **external sources** (scrapers), like the built in TMDB, MusicBrainz, ISBN
and Comic Vine ones.

## Configuration

Declare your plugins in the `scrapers` section of `config.yml`, at the root of the repository:

```yaml
scrapers:
  - id: anime
    placeholder: Anime url
    sources:
      AniDB:
        - tv_show
      MyAnimeList:
        - tv_show
        - movie
    url: http://anime-plugin:9000/scrape
    timeout_seconds: 30
```

| Field | Description |
| --- | --- |
| `id` | Identifier used in the route `/api/external-sources/<id>`. Must not clash with a built in source (`tmdb`, `musicbrainz`, `isbn`, `comic-vine`). |
| `placeholder` | Placeholder of the input field displayed in the interface, also used as the source's display name. |
| `sources` | The websites the endpoint accepts links from, each with the content types it supports: `movie`, `video`, `tv_show`, `music`, `podcast`, `software`, `book`, `live_performance`, `collection`. A single endpoint may serve several websites, dispatching on the link it is given. They are listed in a tooltip next to the input on the upload page once there are several of them, and the source is offered for every content type at least one of them supports. |
| `url` | Endpoint of the plugin. |
| `timeout_seconds` | Optional, defaults to `30`. |

`config.yml` is read once at startup. When the `scrapers` section is absent, no plugin is
registered.

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

The declared endpoint is called with the identifier the user typed as the `url` query parameter,
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
