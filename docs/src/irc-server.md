# IRC Server (Ergo & KiwiIRC)

Arcadia includes integrated IRC chat using [Ergo](https://ergo.chat/) as the IRC server and [KiwiIRC](https://kiwiirc.com/) as the web-based chat client. KiwiIRC is used on the home page and as a help chat for unauthenticated users, but any IRC client can be used to reach ergo.

The integration is optional and runs in dedicated containers enabled via the `irc` or `full` Docker Compose profiles.

---

## Setup Guide

### 1. Copy Configuration Templates

Before starting the IRC service, create copies of the example configuration files:

```bash
cp ergo/ergo-conf.yaml.example ergo/ergo-conf.yaml
cp ergo/ergo.motd.example ergo/ergo.motd
cp kiwiirc/config.json.example kiwiirc/config.json
```

> [!IMPORTANT]
> `ergo/ergo-conf.yaml` and `ergo/ergo.motd` must exist on the host before starting the container, as `compose.yml` mounts them and would otherwise create empty directories.

### 2. Configure Tokens & Enable in `config.yml`

In `.env`, define secure random tokens:

```bash
ARCADIA_ERGO__API_BEARER_TOKEN=your_secure_bearer_token
ARCADIA_ERGO__AUTH_CALLBACK_TOKEN=your_other_secure_token
```

In `config.yml`, uncomment the `ergo:` block to activate IRC features:

```yaml
ergo:
  api_url: http://ergo:8089
```

*(Alternatively, provide `ARCADIA_ERGO__API_URL: http://ergo:8089` in `compose.override.yml` under `backend.environment`).*

> [!NOTE]
> Arcadia's backend uses `api_bearer_token` to provision IRC accounts via Ergo's administrative HTTP API (`/v1/saregister`). When users connect, Ergo verifies their credentials against Arcadia via an auth callback secured by `auth_callback_token`.
> ergo needs the callback_token int the `auth-script` (loaded from the environment in the default setup)

### 3. Adapt `ergo/ergo-conf.yaml` to Your Domain

The template is already tuned for Arcadia, only the lines marked `CHANGE:` need editing:

```yaml
network:
  name: MySite # shown to IRC clients, no spaces

server:
  name: irc.arcadia.example.com # your actual domain

  websockets:
    allowed-origins:
      - "https://arcadia.example.com" # Arcadia frontend origin
```

> [!IMPORTANT]
> In production, always set `allowed-origins` to your frontend origin. An empty list accepts websocket connections from any website.

### 4. Start IRC Services

Launch the IRC daemon and history database:

```bash
docker compose --profile irc up -d
docker compose restart backend
```

Services started:
- **`ergo`**: The IRC daemon. External desktop clients (HexChat, WeeChat) can connect via plain IRC on port `6667`.
- **`ergo_database`**: MariaDB instance storing channel and direct message history.
- **`frontend`**: Caddy automatically routes KiwiIRC assets at `/kiwiirc/` and proxies WebSocket connections (`/webirc/websocket` &rarr; `ergo:8097`).

---

## Production Reverse Proxy & TLS

The webchat needs nothing extra: the main block of [`Caddyfile.example`](https://github.com/Arcadia-Solutions/arcadia/blob/main/Caddyfile.example) serves KiwiIRC and proxies its websocket to Ergo over Arcadia's HTTPS domain.

Desktop clients connect over raw TCP, which Caddy does not proxy. Port `6667` is plain text, so offer TLS (port `6697` by convention) in one of two ways:

- **Ergo terminates TLS:** mount your certificate in the `ergo` container and add a TLS listener in `ergo/ergo-conf.yaml`:

  ```yaml
  server:
    listeners:
      ":6697":
        tls:
          cert: /etc/letsencrypt/live/irc.arcadia.example.com/fullchain.pem
          key: /etc/letsencrypt/live/irc.arcadia.example.com/privkey.pem
  ```

  ```yaml
  # compose.override.yml
  services:
    ergo:
      ports: !override
        - "6697:6697" # replaces the plain 6667 port of compose.yml
      volumes:
        # the whole directory, as the files in live/ are symlinks to archive/
        - /etc/letsencrypt:/etc/letsencrypt:ro
  ```

  Ergo reloads certificates on `SIGHUP` (`docker compose kill -s HUP ergo`), run it after each renewal.

- **A TCP proxy terminates TLS** (nginx `stream`, HAProxy, ...) and forwards to `ergo:6667`. To keep the users' real IPs, enable the PROXY protocol on the proxy, mark the listener with `proxy: true` and add the proxy's IP to `server.proxy-allowed-from`, otherwise every user appears to connect from the proxy:

  ```yaml
  server:
    listeners:
      ":6667":
        proxy: true
    proxy-allowed-from:
      - localhost
      - 172.16.0.0/12 # the proxy's address or network
  ```

In both cases, stop publishing the plain `6667` port on the host (`ports: !override` as above, or `ports: !reset []` when only the TCP proxy reaches Ergo).

---

## Channels & Guest Webchat

In `config.yml`, configure default channels that unauthenticated visitors can join from the login page:

```yaml
frontend:
  irc_webchat_guest_channels:
    - "#help"
```

To support unauthenticated guest chat, ensure `accounts.require-sasl.enabled: false` in `ergo/ergo-conf.yaml`. Keep member-only channels restricted by setting their channel mode to `+r` (registered accounts only).

---

## Local Development with Vite

When running `npm run dev` in the `frontend` directory:
- Vite proxies `/webirc/websocket` to `ws://localhost:8097`.
- Expose port `8097` in [`compose.override.yml`](compose-override.md):
  ```yaml
  services:
    ergo:
      ports:
        - "8097:8097"
  ```
- Build and extract KiwiIRC web assets locally:
  ```bash
  cd frontend
  npm run kiwi:setup
  ```
