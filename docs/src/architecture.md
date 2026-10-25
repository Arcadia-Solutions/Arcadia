# Architecture

## Overview

Arcadia is made of 2 main parts: the site's API and a tracker. The site's API is meant to be used by the frontend, while the tracker is meant to be used by torrent clients (qbittorrent, deluge, etc.).

## Backend

Arcadia's backend is a [REST](https://en.wikipedia.org/wiki/REST) API written in rust with the [actix](https://github.com/actix/actix-web) framework and the [sqlx](https://github.com/launchbadge/sqlx) database driver. It also uses [PostgreSQL](https://www.postgresql.org/) as its database.

### Code Structure

API calls are forwarded to `handlers`, database requests are done by `repositories`, objects are defined by `models`. Directories with those names contain the relevant code.

A swagger for the API is available at `http://localhost:8080/swagger-ui/`

## Frontend

Arcadia's frontend is a [SPA](https://developer.mozilla.org/en-US/docs/Glossary/SPA) written in [TypeScript](https://www.typescriptlang.org/) and uses the [Vue.js](https://vuejs.org/) framework with [PrimeVue](https://primevue.org/) components, [Vite](https://vite.dev/) builds it.

## API Schema Updates

If you make changes to structs that are listed in the swagger or the api routes, you must regenerate the typescript interfaces with this command (from the frontend directory, while the backend is running):

```bash
npm run gen:api
```

It can also work without a running backend, from a spec dumped by the api binary (CI checks the committed client this way):

```bash
cargo run -p arcadia-api -- openapi > /tmp/openapi.json
OPENAPI_SPEC=/tmp/openapi.json npm run gen:api
```
