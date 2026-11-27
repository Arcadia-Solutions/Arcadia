# Upgrading Arcadia

This guide covers updating an existing Arcadia deployment for both **Docker Compose** and **Bare-Metal** installations.

---

## 1. Routine Updates (No Schema Changes)

When pulling updates that do not alter database migrations in `backend/storage/migrations/`:

### Docker:
```bash
git fetch && git pull
docker compose up -d --build
```
The `init_db` container automatically applies any pending incremental migrations and starts the application stack.

### Bare-Metal:
```bash
git fetch && git pull
cargo build -p arcadia-api --release
cargo build -p arcadia_tracker --release
cd frontend && npm install && npm run build && cd ..
```
Restart your backend (`arcadia-api`) and tracker (`arcadia_tracker`) processes or systemd services.

---

## 2. Upgrading Across Schema Changes

Because Arcadia is currently under rapid development, database schema changes are often committed directly to the baseline migration (`backend/storage/migrations/20250312215600_initdb.sql`) rather than distributed as incremental migration scripts.

When upstream commits alter `20250312215600_initdb.sql`, running `sqlx migrate run` (or letting the Docker `init_db` service run) directly against an existing database will fail with a checksum mismatch error.

Use the following procedure to migrate your data into the updated schema.

> [!NOTE]
> In the commands below, replace `arcadia` with your configured database username or database name if you modified defaults in `.env` (Docker) or `config.yml` (bare-metal).

### Step 0: Drop Removed Columns

The dump lists every column explicitly, so a column that the new schema no longer has makes the restore fail. Drop the removed columns from the current database before dumping (`IF EXISTS` makes this safe to run on any version):

- `forum_sub_categories.forbidden_classes`: never enforced, replaced by the `forum_sub_category_forbidden_classes` table

**Docker:**
```bash
docker compose exec -T db psql -U arcadia -d arcadia -c "ALTER TABLE forum_sub_categories DROP COLUMN IF EXISTS forbidden_classes;"
```

**Bare-Metal:**
```bash
psql -U arcadia -d arcadia -c "ALTER TABLE forum_sub_categories DROP COLUMN IF EXISTS forbidden_classes;"
```

### Step 1: Dump Existing Data

Create a data-only SQL dump with explicit column inserts, excluding the `_sqlx_migrations` table:

**Docker:**
```bash
docker compose exec -T db pg_dump -U arcadia -d arcadia --data-only --column-inserts -T _sqlx_migrations > arcadia-data.sql
```

**Bare-Metal:**
```bash
pg_dump -U arcadia -d arcadia --data-only --column-inserts -T _sqlx_migrations > arcadia-data.sql
```

> [!IMPORTANT]
> Verify that `arcadia-data.sql` exists, is non-empty, and is stored in a safe backup location before proceeding.

### Step 2: Pull Latest Commits

```bash
git fetch && git pull
```

### Step 3: Recreate Database and Apply Schema

Drop and recreate the database, then run the updated schema migration:

**Docker:**
```bash
# Stop backend and tracker to release existing database connections
docker compose stop backend tracker

# Drop and recreate the database
docker compose exec -T db dropdb -U arcadia arcadia
docker compose exec -T db createdb -U arcadia arcadia

# Rebuild init_db image with the new migrations and apply schema
docker compose build init_db
docker compose run --rm init_db
```

**Bare-Metal:**
Stop backend and tracker services and then:
```bash
# drop and recreate the database
dropdb -U arcadia arcadia
createdb -U arcadia arcadia
# run the new migrations
cd backend/storage
sqlx migrate run --database-url postgresql://arcadia:your_password@localhost:5432/arcadia
cd ../..
```

### Step 4: Clear Pre-seeded Default Rows

The initial migration seeds default rows (`arcadia_settings`, system `users`, default `user_classes`, `css_sheets`, and initial `forum_*` entries). Truncate these seeded tables before restoring your dump to prevent primary key conflicts:

**Docker:**
```bash
docker compose exec -T db psql -U arcadia -d arcadia -c "
TRUNCATE users, user_classes, css_sheets, arcadia_settings, 
         forum_categories, forum_sub_categories, forum_threads, forum_posts CASCADE;
"
```

**Bare-Metal:**
```bash
psql -U arcadia -d arcadia -c "
TRUNCATE users, user_classes, css_sheets, arcadia_settings, 
         forum_categories, forum_sub_categories, forum_threads, forum_posts CASCADE;
"
```

### Step 5: Restore Data

Restore the dump while temporarily disabling foreign-key triggers (`session_replication_role = 'replica'`). This allows foreign-keyed tables (such as hierarchical `user_classes` or forum relationships) to restore regardless of insert ordering:

**Docker:**
```bash
(echo "SET session_replication_role = 'replica';"; cat arcadia-data.sql; echo "SET session_replication_role = 'default';") | docker compose exec -T db psql -U arcadia -d arcadia -v ON_ERROR_STOP=1
```

**Bare-Metal:**
```bash
(echo "SET session_replication_role = 'replica';"; cat arcadia-data.sql; echo "SET session_replication_role = 'default';") | psql -U arcadia -d arcadia -v ON_ERROR_STOP=1
```

### Step 6: Recompile and Restart

Start the stack with updated code and dependencies:

**Docker:**
```bash
docker compose up -d --build
```

**Bare-Metal:**
```bash
cargo build -p arcadia-api --release
cargo build -p arcadia_tracker --release
cd frontend && npm install && npm run build && cd ..
```
Restart your backend (`arcadia-api`) and tracker (`arcadia_tracker`) processes or systemd services.
