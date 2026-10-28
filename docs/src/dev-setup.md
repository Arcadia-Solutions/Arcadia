# Developer Setup

## Development Containers (Optional)

<div class="warning">

If you don't want to install another toolchain on your system, You can also use [devcontainers](https://containers.dev/) instead.
If you don't know, think of isolated minimal virtual machines that come with the tools required to build Arcadia (or anything else really).

If you have Docker (recommended!) installed and use [Visual Studio Code](https://code.visualstudio.com/), all you need is to have the [Dev Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension installed and "reopening your folder in a container".
You can find that option in your Command Palette, or by clicking on the new status bar item in the left bottom corner.
</div>

You can also use GitHub Codespaces to build Arcadia in the cloud without having to download anything but streams of text, although it's not as free as a local dev container.

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/Arcadia-Solutions/arcadia?quickstart=1)

It isn't required to use them but can be useful in some cases (especially if you're using an immutable OS).

## Required Tools

You need these to make meaningful contributions to Arcadia, outside the cases of documentation for example.

- [Node.js & npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (version 1.88.0 and higher)

## Recommended Tools

- [Prettier](https://prettier.io) for proper formatting of the frontend's code.
- [sqlx-cli](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) for managing database related stuff, including migrations.
- [Docker](https://docs.docker.com/desktop/setup/install) for setting up dependencies. Optional but HIGHLY recommended!
- [Insomnia](https://github.com/Kong/insomnia/) for testing the backend's API. You could also use any other client if you want.

## Configuration Setup

Everything is configured by a single `config.yml` at the root of the repository. A quick way to
get started is `cp config.example.yml config.yml`: that sample documents every key and is the
reference for what each one does. Generate the secrets left empty in it (`api.jwt_secret` and `tracker.api_key`) with
`openssl rand -hex 32`, the services refuse to start without them.

### Compile-Time Requirement: `DATABASE_URL`

The `sqlx` query macros check the queries against a real database **at compile time**, and `sqlx`
only reads `DATABASE_URL`. It is needed for `cargo build`, `cargo clippy`, and `cargo sqlx prepare`,
never by the running services.

If you already created `.env` from `example.env`, append `DATABASE_URL` to it so existing credentials are not overwritten:

```bash
echo 'DATABASE_URL=postgresql://arcadia:password@localhost:5432/arcadia' >> .env
```

If you are running the database with Docker, port 5432 is not exposed to the host by default.
See the [database port mapping overrides](compose-override.md#2-expose-internal-databaseredis-ports-for-host-debugging-development)
to expose it with `compose.override.yml`.

Docker builds don't need it, they build with `SQLX_OFFLINE=true` against the committed `.sqlx`
caches.

## Building and Running

### API

```bash
# Build the backend
cargo build -p arcadia-api

# Run the backend binary
./target/debug/arcadia-api

# For optimized builds
cargo build -p arcadia-api --release
./target/release/arcadia-api
```

### Frontend

```bash
cd frontend

# Install dependencies
npm install

# Build and run development server
npm run dev

# For production build
npm run build
```

## Development Workflow

### Backend Development

```bash
# For development with auto-rebuild on changes
cd backend/api
cargo run

# Build and test
cargo build -p arcadia-api
cargo test

# Code quality checks
cargo clippy --fix --allow-dirty
cargo fmt --all
```

### Frontend Development

```bash
cd frontend

# Development server with hot reload
npm run dev

# Run tests
npm run test:unit

# Lint and format
npm run lint
npm run format
```

#### Optional: IRC & KiwiIRC Webchat in Local Development

When running `npm run dev`, Vite includes proxying for Ergo's WebSocket endpoint (`/webirc/websocket` &rarr; `ws://localhost:8097`) and serves static KiwiIRC webchat assets under `/kiwiirc/`.

Both components are completely optional during development. If KiwiIRC is not built, Vite displays an informative placeholder in the chat drawer. If Ergo is not running, WebSocket connection failures are handled silently so the Vite dev server remains stable.

To enable full IRC and KiwiIRC functionality locally:

1. **Expose Ergo's WebSocket port in Docker**:
   By default, port `8097` is not exposed on the host. Create or add to `compose.override.yml` at the repository root:
   ```yaml
   services:
     ergo:
       ports:
         - "8097:8097"
   ```
   Then start Ergo:
   ```bash
   docker compose --profile irc up -d ergo
   ```
   *(See also [Docker Compose Overrides](compose-override.md)).*

2. **Populate `kiwiirc/dist`**:
   KiwiIRC assets are git-ignored. You can automatically build and extract them using Docker

   ```bash
   cd frontend
   npm run kiwi:setup
   ```
   This builds KiwiIRC using Docker with the repository's pinned commit and extracts the compiled assets into `kiwiirc/dist/`.

> [!NOTE]
> Vite dynamically intercepts `/kiwiirc/static/config.json` (falling back to `config.json.example` if not present) and `/kiwiirc/static/plugins/arcadia-plugin.js` to serve them directly from the `kiwiirc/` directory in the repository. You can modify either file and refresh the browser without rebuilding KiwiIRC.


## Common Docker Commands

### Auto-rebuild with Compose Watch

For live development, Compose Watch automatically rebuilds images or syncs frontend files on source changes:

```bash
docker compose up --watch
```

Or when running attached without `-d`, press <kbd>W</kbd> to enable watch mode.

### Exporting Test Data

If you added new test data in your local container and wish to update the repository fixtures:

```bash
docker compose exec -T db pg_dump -U arcadia -d arcadia --data-only --inserts --column-inserts > backend/storage/migrations/fixtures/fixtures.sql && sed -i '/SELECT pg_catalog.set_config(\x27search_path\x27, \x27\x27, false);/d' backend/storage/migrations/fixtures/fixtures.sql
```

1 line generated by `pgdump` must be removed as it prevents the `collage_entry` fixtures from being inserted (the trigger somehow can't be interprted). If someone has an explanation, please let us know/open a PR!

## Manual Database Setup (if needed)

Arcadia automatically runs migrations on launch (`init_db` container), but if you need to manually run migrations against a running database:

```bash
cargo install sqlx-cli
DATABASE_URL=postgresql://arcadia:password@localhost:5432/arcadia cargo sqlx database setup
```

`sqlx-cli` only reads `DATABASE_URL` (configured in `.env`), it does not read `config.yml`.
Make sure the database port is exposed.
