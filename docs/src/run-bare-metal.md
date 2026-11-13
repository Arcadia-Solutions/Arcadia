# Manual / Bare-Metal Installation

This page explains how to install and run Arcadia directly on your system without Docker containers.

## Prerequisites

Before starting, ensure you have the following installed:

- **PostgreSQL** - Database server
- **Redis** - Cache for the auth
- **Rust & Cargo** - Required to build the backend
- **Node.js & npm** - Required to build the frontend
- **Git** - To clone the repository

For development tool installation instructions, see the [Developer Setup](dev-setup.md) guide.

## Quick Start

1. Clone the repository and navigate to it:
   ```bash
   git clone https://github.com/Arcadia-Solutions/arcadia.git
   cd arcadia
   ```
2. Set up PostgreSQL database and run migrations
3. Set up Redis server
4. Configure `config.yml` (uncomment `database` and `redis`, adapt internal URLs)
5. Configure and run the backend (`arcadia-api`)
6. Configure and run the frontend (`cd frontend && npm run dev`)
7. Configure and run the tracker (`arcadia_tracker`)

## Database Setup

### 1. Install PostgreSQL

Install PostgreSQL on your system:

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib
```

**macOS:**
```bash
brew install postgresql
brew services start postgresql
```

**Windows:**
Download and install from the [PostgreSQL official website](https://www.postgresql.org/download/windows/).

### 2. Create Database and User

Connect to PostgreSQL and create the database:

```bash
# Connect as postgres user
sudo -u postgres psql

# Or on Windows/macOS:
psql -U postgres
```

In the PostgreSQL shell:
```sql
-- Create user
CREATE USER arcadia WITH PASSWORD 'your_secure_password';

-- Create database
CREATE DATABASE arcadia OWNER arcadia;

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE arcadia TO arcadia;

-- Exit
\q
```

### 3. Run Database Migrations

Install `sqlx-cli` and run migrations:

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features native-tls,postgres

# Navigate to storage directory
cd backend/storage

# Run migrations
sqlx migrate run --database-url postgresql://arcadia:your_secure_password@localhost:5432/arcadia

# Return to repository root
cd ../..
```

`sqlx-cli` only reads `--database-url` or `DATABASE_URL`, it does not know about `config.yml`.
Use the credentials matching your PostgreSQL setup.

*(If you get a "Could not find directory of OpenSSL installation" error, install `pkg-config` and `libssl-dev` / `openssl-devel`).*

#### Optional: Seed Development Fixtures
If you want to populate sample categories, users, and torrents for testing:

```bash
psql -U arcadia -d arcadia -f backend/storage/migrations/fixtures/fixtures.sql
```
Default test credentials: `picolo` / `test`.

For a clean production installation, skip this step and see [Bootstrapping the Administrator](#bootstrapping-the-administrator) below.

## Redis Setup

Arcadia uses Redis for session management and caching.

**Ubuntu/Debian:**
```bash
sudo apt-get install redis-server
sudo systemctl enable --now redis-server
```

**macOS:**
```bash
brew install redis
brew services start redis
```

By default on local systems, Redis binds to `127.0.0.1:6379` without a password. If you configure a password in Redis (`requirepass <password>` in `/etc/redis/redis.conf`), make sure to specify it in `config.yml`.

## Configuration for Bare Metal

Create `config.yml` from the example (see the [Configuration Reference](configuration.md) for details on secrets and environment variable overrides):

```bash
cp config.example.yml config.yml
```

> [!IMPORTANT]
> `config.example.yml` defaults to Docker hostnames. For a bare-metal installation, review settings marked `# Bare-metal:` and `# Production:` in `config.yml`, including:
>
> 1. **Uncomment `database:` and `redis:`**:
>    ```yaml
>    database:
>      host: 127.0.0.1
>      port: 5432
>      user: arcadia
>      password: your_secure_password
>      name: arcadia
>
>    redis:
>      host: 127.0.0.1
>      port: 6379
>      password: ""
>    ```
> 2. **Adjust internal URLs**:
>    - In `tracker:`, set `url_internal: http://localhost:8081` (not `http://tracker:8081`).
> 3. **Set production secrets**:
>    - `api.jwt_secret` and `tracker.api_key`.

## Backend Setup

### Build and Run

From the repository root:

```bash
cargo run -p arcadia-api --release
```

The backend server (including the integrated periodic task scheduler) will start and listen on port `8080` (`http://localhost:8080`).

## Frontend Setup

### Build and Run

Navigate to the `frontend` directory, install dependencies, and start the development server:

```bash
cd frontend
npm install
npm run dev
```

The frontend will be accessible at `http://localhost:5173`. Vite proxies `/api/` requests to the backend at `http://localhost:8080`.

## Tracker Setup

### Build and Run

From the repository root in a separate terminal:

```bash
cargo run -p arcadia_tracker --release
```

The BitTorrent tracker will start listening on port `8081`.

## Bootstrapping the Administrator

The database schema comes with an `admin` user having all permissions, its password is `arcadia`. Log in with it at `http://localhost:5173/login` and change its password right away from the user settings.

## Upgrading

For routine updates that do not alter the database schema:

```bash
git fetch && git pull
cargo build -p arcadia-api --release
cargo build -p arcadia_tracker --release
cd frontend && npm install && npm run build && cd ..
```

Restart your backend and tracker services.

> [!WARNING]
> If upstream commits modify database schema migrations (`initdb.sql`), follow the [Schema Migration Upgrade Guide](upgrade.md#2-upgrading-across-schema-changes) for the data dump and schema migration procedure.

## Troubleshooting

### Database Issues

**PostgreSQL not running:**
```bash
# Ubuntu/Debian
sudo systemctl start postgresql
sudo systemctl enable postgresql

# macOS
brew services start postgresql
```

**Connection errors:**
- Verify PostgreSQL is running on port 5432
- Check that the database and user exist
- Ensure the `database` section of `config.yml` is correct

### Build Issues

**Backend API build fails:**
- Install system dependencies listed above
- Update Rust: `rustup update`
- Clear build cache: `cargo clean`

**Frontend build fails:**
- Check Node.js version compatibility
- Clear npm cache: `npm cache clean --force`
- Delete `node_modules` and run `npm install` again

### Runtime Issues

**Backend API won't start:**
- Check the database connection
- Verify the values in `config.yml`
- Ensure migrations have been run

**Frontend can't connect to backend API:**
- Verify the backend is running on the correct port
- Check `frontend.api_base_url` in `config.yml`, and restart the frontend: the section is inlined
  in the bundle at build time

## Environment Variable Overrides

Any value in `config.yml` can be overridden via environment variables without editing the file.
See the [Configuration](configuration.md#environment-variables) section for the full `ARCADIA_<SECTION>__<KEY>` syntax and common examples.

## Stopping Arcadia

To stop Arcadia:
1. Stop the frontend with `Ctrl+C` in its terminal
2. Stop the tracker with `Ctrl+C` in its terminal and wait for its graceful shutdown
3. Stop the backend API with `Ctrl+C` in its terminal
4. Optionally stop PostgreSQL if you don't need it for other applications
