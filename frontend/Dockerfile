ARG NODE_VERSION=22
ARG WITH_KIWI=true

# ── 1. Arcadia Frontend Build ───────────────────────────
FROM node:${NODE_VERSION}-slim AS build-frontend

WORKDIR /home/node/app/frontend

# The build context is the repository root.
COPY frontend/ .

RUN --mount=type=cache,target=/root/.npm \
    npm ci --verbose --no-audit

# The frontend is configured by the `frontend` section of the root configuration file, which vite
# inlines in the bundle at build time. It is mounted as a secret rather than copied, so that the
# rest of it (jwt secret, database password, api keys) does not end up in an image layer.
RUN --mount=type=secret,id=arcadia_config,target=/home/node/app/config.yml \
    npx vite build

# ── 2a. Stub stage (Selected when WITH_KIWI=false) ──────
FROM caddy:2-alpine AS kiwi-false
WORKDIR /kiwi/dist

# ── 2b. Full KiwiIRC build (Selected when WITH_KIWI=true)
FROM node:${NODE_VERSION}-slim AS kiwi-true
WORKDIR /kiwi
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
ARG KIWIIRC_COMMIT=0416529fda6ddf8ab5559796ec69b49e07bd798f
RUN git init && git remote add origin https://github.com/kiwiirc/kiwiirc.git \
    && git fetch --depth 1 origin ${KIWIIRC_COMMIT} && git checkout ${KIWIIRC_COMMIT} \
    && corepack enable && yarn install --frozen-lockfile && yarn build
COPY kiwiirc/config.json* /kiwi/dist/static/
RUN [ -f /kiwi/dist/static/config.json ] || cp /kiwi/dist/static/config.json.example /kiwi/dist/static/config.json
COPY kiwiirc/arcadia-plugin.js /kiwi/dist/static/plugins/arcadia-plugin.js

# ── 2c. Dynamic Selector ────────────────────────────────
FROM kiwi-${WITH_KIWI} AS kiwi-assets

# ── 3. Final Caddy Runtime ──────────────────────────────
FROM caddy:2-alpine

COPY --from=build-frontend /home/node/app/frontend/dist/ /srv/arcadia/
COPY --from=kiwi-assets /kiwi/dist/ /srv/kiwiirc/
COPY frontend/Caddyfile /etc/caddy/Caddyfile
