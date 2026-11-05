# Image Hosting (Elysia)

Arcadia uses [Elysia](https://github.com/Arcadia-Solutions/Elysia) to host images on your site.

The service is optional and runs in dedicated containers enabled via the `images` or `full` Docker Compose profiles (see the Elysia services in `compose.yml`).

Uploaded images are attributed to the Arcadia user who uploaded them: Arcadia uploads on their behalf, so in Elysia each image belongs to the same account the user gets when logging in to Elysia with Arcadia.

## Setup Guide

### 1. Get Elysia and Start Containers

Elysia has no published image: it is built from the `elysia` git submodule of the Arcadia repository.

```bash
git submodule update --init elysia
cp elysia/config.example.yml elysia/config.yml
```

After pulling a new Arcadia version, run `git submodule update elysia` again so Elysia follows it.

In `.env`, define a secure password for Elysia's database:

```bash
ELYSIA_DB_PASSWORD=your_secure_elysia_password
```

In `elysia/config.yml`, set `server.public_url` to where users reach Elysia: Arcadia's frontend serves it under `/elysia` (e.g. `https://arcadia.example.com/elysia`, or `http://localhost:5173/elysia` locally), before the first upload, since links already stored in Arcadia keep the old value, and set `auth.jwt_secret` to a long random value (e.g. `openssl rand -hex 32`).

### 2. Connect Elysia and Arcadia

Arcadia is Elysia's login provider, and uploads to Elysia with the same client credentials. Generate a client secret (e.g. `openssl rand -hex 32`) and set it on both sides.

In `elysia/config.yml`, uncomment `auth.sso`:

```yaml
auth:
  sso:
    name: "Arcadia"
    issuer: "https://arcadia.example.com/api/oauth"
    client_id: "elysia"
    client_secret: "the_client_secret"
    redirect_uri: "https://arcadia.example.com/elysia/auth/sso/callback"
    status_endpoint: "https://arcadia.example.com/api/oauth/users/{sub}/status"
```

The issuer must be reachable from the `elysia` container.

In Arcadia's `config.yml`, configure the `oidc` section with the same values, and point `image_host` at Elysia's backend:

```yaml
oidc:
  issuer: https://arcadia.example.com/api/oauth
  client_id: elysia
  client_secret: the_client_secret
  redirect_uri: https://arcadia.example.com/elysia/auth/sso/callback

image_host:
  elysia_url: http://elysia:8090
  # Automatically rehost posters and covers retrieved by scrapers (TMDB, MusicBrainz)
  rehost_external_images: true
```

Uploads need both `image_host.elysia_url` and `oidc`.

Launch the image hosting services and restart the backend to apply the configuration:

```bash
docker compose --profile images up -d --build
docker compose restart backend
```

### 3. Enable Image Uploader in Staff Dashboard

By default, the frontend drag-and-drop uploader is disabled. To activate it:

1. Log in with an account having administrator permissions.
2. In the top navbar, navigate to **Staff Dashboard** &rarr; **Arcadia Settings**.
3. Enable **Display image upload drag and drop** and save changes.

The image upload widget will now appear when creating or editing torrents, editions, and artists. If you enforce an **Approved Image Hosts** whitelist in site settings, add Arcadia's own domain (e.g. `arcadia.example.com`) to the list, since Elysia is served under it.

## Production Reverse Proxy & HTTPS

Elysia is served under `/elysia` of Arcadia's domain by the main block of [`Caddyfile.example`](https://github.com/Arcadia-Solutions/arcadia/blob/main/Caddyfile.example), so it needs no domain or port of its own.

## Migrating from Chevereto

Elysia ships an importer that copies every Chevereto image, keeping its uploader, and rewrites the Chevereto links stored in Arcadia's database: see [Importing from Chevereto](https://github.com/Arcadia-Solutions/Elysia/blob/main/backend/README.md#importing-from-chevereto).

When upgrading an existing deployment:

- Keep the Chevereto block of your `Caddyfile` enabled until the import has rewritten the links, otherwise the existing image links stop resolving.
- Add `elysia_storage` to `backup.volumes` in `config.yml` (docker mode), otherwise the backups hold Elysia's database but not its images, then recreate `backup_cron` so it mounts the new volume: `docker compose --profile backup up -d --force-recreate backup_cron`.

## Chevereto (deprecated)

[Chevereto](https://chevereto.com/) is still supported but deprecated. Its containers (`chevereto_php`, `chevereto_database`) start with the `chevereto` and `full` profiles. When `image_host.elysia_url` and `oidc` are set, the Chevereto settings are ignored.

1. In `.env`, set `CHEVERETO_DB_PASS` and `CHEVERETO_DB_ROOT_PASS`, then run `docker compose --profile chevereto up -d`.
2. Open `http://localhost:8083`, complete the installation wizard, then generate an **API v1 key** in **Dashboard** &rarr; **Settings** &rarr; **API**.
3. In `config.yml`, configure the `image_host:` section and restart the backend:

   ```yaml
   image_host:
     chevereto_api_url: http://chevereto_php/api/1/upload
     chevereto_api_key: your_generated_chevereto_api_key
     rehost_external_images: true
   ```

   *(Alternatively, inject the API key via environment variable: `ARCADIA_IMAGE_HOST__CHEVERETO_API_KEY=your_key`).*

All uploads are made with the same API key, so Chevereto does not know which user uploaded an image. In production, set `CHEVERETO_HOSTNAME` and `CHEVERETO_HTTPS` in `compose.override.yml` and use the Chevereto block of `Caddyfile.example` instead of the Elysia one.
