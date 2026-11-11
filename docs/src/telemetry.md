# Monitoring (OpenTelemetry & Grafana)

Arcadia includes deep observability instrumentation with [OpenTelemetry](https://opentelemetry.io/). The backend, BitTorrent tracker, and periodic background scheduler emit structured metrics and distributed traces via OTLP gRPC. The [backup](./backup.md#telemetry) job also reports per-run metrics (over OTLP/HTTP) when the same endpoint is set.

## Architecture

When running with the `telemetry` or `full` profile, Docker Compose provisions:
- **`otel-lgtm`**: A unified container bundling Grafana (port `3000`), Prometheus/Mimir (metrics), Loki (logs), and Tempo (traces) with an integrated OTLP gRPC collector on port `4317`.
- **`hostmetrics`**: An OpenTelemetry Collector container gathering system-level host metrics (CPU, RAM, disk I/O, network) from the Docker host.

## Setup Guide

### 1. Set Grafana Password in `.env`

In `.env`, define your secure administrator password for the Grafana UI:

```bash
GF_SECURITY_ADMIN_PASSWORD=your_secure_grafana_password
```

### 2. Start the Telemetry Containers

Launch the monitoring services:

```bash
docker compose --profile telemetry up -d
```

### 3. Enable Telemetry Export in Arcadia

In `config.yml`, configure the `telemetry:` section to export OTLP data:

```yaml
telemetry:
  otlp_endpoint: http://otel-lgtm:4317
```

Restart `backend` and `tracker` to begin exporting data:

```bash
docker compose restart backend tracker
```

If [Elysia](image-host.md) runs too, set the same section in `elysia/config.yml` and restart it:

```bash
docker compose restart elysia_backend
```

### 4. Open the Dashboards

Open `http://localhost:3000` and sign in as `admin` with the value of `GF_SECURITY_ADMIN_PASSWORD` (defaults to `arcadia`). Grafana loads the dashboards at startup: **Arcadia and Host metrics** from [`opentelemetry/sample-dashboard.json`](https://github.com/Arcadia-Solutions/arcadia/blob/main/opentelemetry/sample-dashboard.json) and, when the `elysia` submodule is checked out, **Elysia** from Elysia's own `opentelemetry/dashboard.json`. Grafana picks up edits to these files within seconds.

The Arcadia dashboard provides real-time visibility into:
- API endpoint request rates, response latency percentiles (p50, p95, p99), and HTTP status codes.
- BitTorrent tracker announce and scrape request throughput.
- Peer connection tracking, caching ratios, and active peer counts.
- Periodic background tasks: execution intervals, durations, and rows affected.
- Backup runs: last-run status, wall-clock and restic duration, data added, and files processed.
- Host CPU, memory pressure, and network throughput.

## Production Reverse Proxy & HTTPS

To access Grafana securely in production, enable the Grafana block in `Caddyfile` (pre-configured in [`Caddyfile.example`](https://github.com/Arcadia-Solutions/arcadia/blob/main/Caddyfile.example)), and uncomment its `@allowed` matcher to restrict the dashboards to your management IPs.

`compose.yml` also publishes Grafana on port `3000` of the host, which bypasses Caddy's HTTPS and IP filter. Once Caddy serves it, stop publishing that port in `compose.override.yml`:

```yaml
services:
  otel-lgtm:
    ports: !reset []
```

The OTLP collector (`4317`) is only reachable inside the Docker network: keep it that way, it accepts data without authentication.
