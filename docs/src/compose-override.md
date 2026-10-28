# Customizing with Compose Override

Docker Compose automatically detects and merges `compose.override.yml` with `compose.yml`. Use this to adjust ports, volumes, or environment variables without modifying the version-controlled `compose.yml`.

The repository provides ready-to-use example configurations tracked directly in git:
- [`compose.override.yml.example`](https://github.com/Arcadia-Solutions/arcadia/blob/main/compose.override.yml.example): Configures port forwarding (80/443), Caddy volume mounts, and service tweaks.
- [`Caddyfile.example`](https://github.com/Arcadia-Solutions/arcadia/blob/main/Caddyfile.example): Complete reverse proxy configuration routing the Web UI, API, IRC, BitTorrent Tracker, Chevereto image host, and Grafana monitoring.

To get started:
```bash
cp compose.override.yml.example compose.override.yml
cp Caddyfile.example Caddyfile
```

### 1. Production Setup: Exposing Services via Caddy

In production, you want Caddy to terminate HTTPS and obtain automatic Let's Encrypt certificates for your public domains, proxying each container over Docker's internal network:

```yaml
services:
  frontend:
    ports:
      - "80:80"
      - "443:443"
      - "443:443/udp"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config

volumes:
  caddy_data:
  caddy_config:
```

### 2. Development Setup: Exposing Ports for Host Debugging
```yaml
services:
  db:
    ports:
      - "5432:5432"
  redis:
    ports:
      - "6379:6379"
  backend:
    ports:
      - "8080:8080"
```

### 3. Override configuration with environment variables
You can override any setting from `config.yml` using `environment:` blocks:
```yaml
services:
  backend:
    environment:
      ARCADIA_API__LOG_LEVEL: "debug,sqlx=debug"
      ARCADIA_API__JWT_SECRET: "output-of-openssl-rand-hex-32"
  tracker:
    environment:
      ARCADIA_TRACKER__NUMWANT: "30"
```
