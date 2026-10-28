# Configuration

The whole project is configured by a **single `config.yml` file at the root of the repository**:
the backend, tracker, and frontend build all read it. It is git-ignored. Some of those values are overridden by the Environment Variables in `.env` at runtime

```bash
cp config.example.yml config.yml
cp example.env .env
```

[`config.example.yml`](https://github.com/Arcadia-Solutions/arcadia/blob/main/config.example.yml) documents every key and serves as the reference. Settings are annotated with tags for each environment:
- `# Production:` Settings that must be changed before deploying publicly (secrets, domain URLs, SMTP).
- `# Bare-metal:` Settings requiring host-specific adjustments when running without Docker.
- `# Development:` Settings useful during local testing (e.g. verbose logging).
- `# Optional:` Optional integrations and external plugins.

[`.env`](https://github.com/Arcadia-Solutions/arcadia/blob/main/example.env) is used by Docker Compose to configure passwords and tokens shared across multiple services (such as PostgreSQL, Redis, Chevereto, and Ergo IRC).

For a local test with Docker Compose, the default credentials work out of the box, except `api.jwt_secret` and `tracker.api_key` which must be generated (`openssl rand -hex 32`): services refuse to start with a secret shorter than 32 characters. For a [production environment](run-docker.md#production-deployment), change all secrets and URLs.

## Environment Variables

Any setting in `config.yml` can be overridden via environment variables using the naming convention:
`ARCADIA_<SECTION>__<KEY>` (single underscore after `ARCADIA_`, double underscore `__` between sections and keys). Environment variables always take precedence over values in `config.yml`.

> [!NOTE]
> **Docker Compose vs. Host Environment**: Docker Compose uses `.env` to interpolate `${VARIABLE}` expressions defined in `compose.yml`. Arbitrary `ARCADIA_<SECTION>__<KEY>` variables placed in `.env` are **not** automatically forwarded into containers unless they are explicitly declared under `environment:` in `compose.yml` or added via [`compose.override.yml`](compose-override.md#3-override-configuration-with-environment-variables).

### Common Environment Variable Overrides

| Setting | YAML Key | Environment Variable |
| :--- | :--- | :--- |
| Database password | `database.password` | `ARCADIA_DATABASE__PASSWORD=secret` |
| Database host | `database.host` | `ARCADIA_DATABASE__HOST=db` |
| Redis password | `redis.password` | `ARCADIA_REDIS__PASSWORD=secret` |
| Redis host | `redis.host` | `ARCADIA_REDIS__HOST=redis` |
| API host & port | `api.host`, `api.port` | `ARCADIA_API__HOST=0.0.0.0`, `ARCADIA_API__PORT=8080` |
| Tracker host & port | `tracker.host`, `tracker.port` | `ARCADIA_TRACKER__HOST=0.0.0.0`, `ARCADIA_TRACKER__PORT=8081` |
| JWT Secret | `api.jwt_secret` | `ARCADIA_API__JWT_SECRET=supersecret` |
| Tracker API Key | `tracker.api_key` | `ARCADIA_TRACKER__API_KEY=anothersecret` |

## Site Customization & Theming

For configuring the site logo, favicon, custom landing pages, and unauthenticated CSS/JS stylesheets, see [Customization & Theming](customization.md).

## Optional Integrations

Some services bundled with Arcadia require additional configuration:
- **IRC Server (Ergo & KiwiIRC)**: See [IRC Server](irc-server.md).
- **Image Hosting (Chevereto)**: See [Image Host](image-host.md).
- **Telemetry & Monitoring**: See [OpenTelemetry](telemetry.md).
