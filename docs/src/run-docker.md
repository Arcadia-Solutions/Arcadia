# Docker Deployment

This guide will help you get Arcadia running quickly using Docker Compose.

## Prerequisites

- Docker and Docker Compose installed
- Git (to clone the repository)

<div class="warning">

If running `docker compose` doesn't work, you may have an older version of the docker cli installed and may need to use `docker-compose` instead.

Also don't forget to use `sudo` if you aren't in the `docker` group!
</div>

## Quick Setup

1. **Copy [Configuration Files](configuration.md)**

    ```bash
    cp config.example.yml config.yml
    cp example.env .env
    ```

    Docker Compose automatically injects inter-container networking and credentials via environment variables from `.env`, so no manual changes to `config.yml` are required for a local test run.

2. **Start Services**

    ```bash
    docker compose up -d
    ```
    
    This starts the core services:
    - PostgreSQL database (`db`) and automatic schema migrations (`init_db`)
    - Redis cache (`redis`)
    - Backend API and periodic task runner (`backend`)
    - BitTorrent tracker (`tracker`, listening on host port `8081`)
    - Frontend UI and reverse proxy (`frontend`, powered by Caddy on host port `5173`)

3. **Database Initialization & Initial User Setup**

    Choose one of the following approaches depending on your deployment:

   **Option A: Development (Load Sample Fixtures)**
    Populate the database with demo data:
    
    ```bash
    docker compose exec -T db psql -U arcadia -d arcadia < backend/storage/migrations/fixtures/fixtures.sql
    ```
    
    Credentials of the user with all permissions:
    - **Username**: `picolo`
    - **Password**: `test`

    **Option B: Production (Clean Database & First Admin Setup)**
    If you do not load `fixtures.sql`, the database initializes cleanly with schema migrations alone. To set up your first administrator:
    
    1. Navigate to `http://localhost:5173/register` (or your domain) and register your desired username and password.
    2. By default, newly registered users have the `newbie` class with zero administrative permissions. Promote your account to have full administrator permissions:
       ```bash
       docker compose exec -it db psql -U arcadia -d arcadia -c "UPDATE users SET permissions = enum_range(NULL::user_permissions_enum) WHERE username = 'YOUR_USERNAME';"
       ```
    3. Log in to the web interface to access site administration and create user classes.

4. **Access the Application**
    - Frontend Web UI: `http://localhost:5173`
    - Backend API: `http://localhost:5173/api/` (proxied internally via Caddy)
    - Tracker Announce: `http://localhost:8081/announce/<passkey>` (legacy fallback: `http://localhost:8081/<passkey>/announce`)

## Production Deployment

By default, Docker Compose uses development credentials (`arcadia` / `password`). For production deployments:

1. **Set Strong Passwords in `.env`**:
   ```bash
   cp example.env .env
   ```
   Edit `.env` and replace all placeholder passwords (`ARCADIA_DATABASE__PASSWORD`, `ARCADIA_REDIS__PASSWORD`, etc.) with strong random values (e.g. generated via `openssl rand -hex 32`).

2. **Configure URLs and Secrets in `config.yml`**:
   Update settings marked `# Production:` in [`config.example.yml`](https://github.com/Arcadia-Solutions/arcadia/blob/main/config.example.yml) (`api.jwt_secret`, `tracker.api_key`, public URLs, and `smtp:`). See also the [Configuration Reference](configuration.md).

Docker Compose automatically propagates credentials from `.env` across the stack:
- **`db`**: Configures PostgreSQL user, password, database, and healthcheck.
- **`init_db`**: Injects `DATABASE_URL` for running schema migrations.
- **`redis`**: Configures Redis server password authentication (`--requirepass`).
- **`backend` & `tracker`**: Injects database and Redis credentials via environment variables, overriding values from `config.yml`. Leave the `database:` and `redis:` sections commented out in `config.yml` (any values placed there are overridden and ignored).

> [!NOTE]
> PostgreSQL only uses `POSTGRES_PASSWORD` when initializing a new database cluster. If you change `ARCADIA_DATABASE__PASSWORD` on an existing installation after the `db_data` volume has already been initialized, you must also update the password inside PostgreSQL:
> ```bash
> docker compose exec -it db psql -U arcadia -d arcadia -c "ALTER USER arcadia WITH PASSWORD 'new_password';"
> ```

### Reverse Proxy & HTTPS (Production)

The `frontend` container serves the static frontend files and proxies access to all other containers using [Caddy](https://caddyserver.com/).  
To make Arcadia accessible via HTTPS with automatic Let's Encrypt certificates across all services (Web UI, API, BitTorrent tracker, Chevereto image host, and Grafana monitoring), copy the tracked production templates:

```bash
cp compose.override.yml.example compose.override.yml
cp Caddyfile.example Caddyfile
```

Edit `Caddyfile` with your domain names, and set `api.reverse_proxy_client_ip_header_name` and `tracker.reverse_proxy_client_ip_header_name` to `X-Forwarded-For` in `config.yml`, otherwise the ip address of Caddy is recorded instead of the one of the users. See the [Compose Override Guide](compose-override.md) for full routing and service details.

## Upgrading

For routine updates:

```bash
git fetch && git pull
docker compose up -d --build
```

The `init_db` container automatically runs pending incremental database migrations before the backend starts.

> [!WARNING]
> Because Arcadia is under rapid development, database schema changes are often committed directly to the baseline migration (`backend/storage/migrations/20250312215600_initdb.sql`) rather than distributed as incremental migrations. When this happens, `init_db` will fail with an SQLx checksum mismatch error.
> When pulling updates with baseline schema changes, follow the [Schema Migration Upgrade Guide](upgrade.md#2-upgrading-across-schema-changes) to dump, recreate, and restore your database.

## Troubleshooting

- If services fail to start, check logs with: `docker compose logs [service-name]`
- To rebuild images: `docker compose build`
- To reset everything: `docker compose down -v && docker compose up -d`
