FROM lukemathwalker/cargo-chef:latest-rust-1.94 AS base

FROM base AS planner

WORKDIR /app

COPY ./backend ./backend
COPY ./shared ./shared
COPY ./tracker/arcadia_tracker ./tracker/arcadia_tracker
# Cargo.lock has to be part of the recipe. Without it cargo re-resolves the dependency graph
# on every build, so the recipe (and with it the whole cooked dependency cache) is invalidated
# as soon as any transitive crate publishes a new version.
COPY Cargo.toml Cargo.lock ./

# Creates the manifest for dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder

WORKDIR /app

# Only needed to satisfy the `[patch.crates-io]` entry in the workspace manifest.
COPY --from=planner /app/backend/api/vendored /app/backend/api/vendored/
COPY --from=planner /app/recipe.json recipe.json
COPY .cargo .cargo

# Set to "false" to build without optimizations, for faster builds.
ARG PRODUCTION_BUILD=true

# Caches the target directory for intermediate build artifacts.
# `cargo chef cook` downloads, builds, and caches all of the dependencies for future builds.
# Both cook and build are scoped to `arcadia_tracker`: without `--bin` cargo selects every
# default workspace member, so this image was also building arcadia-api and the rest of the
# backend crates.
RUN --mount=type=cache,id=arcadia-tracker-target,target=/app/target,sharing=locked \
    --mount=type=cache,id=arcadia-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    if [ "$PRODUCTION_BUILD" = "true" ]; then \
    cargo chef cook --bin arcadia_tracker --release --recipe-path recipe.json; \
    else \
    cargo chef cook --bin arcadia_tracker --recipe-path recipe.json; \
    fi

# Only the Rust sources. A blanket `COPY . .` also pulled in frontend/, docs/ and media/, so
# editing anything there invalidated the layer below and re-linked the binary for nothing.
COPY Cargo.toml Cargo.lock rust-toolchain ./
COPY ./backend ./backend
COPY ./shared ./shared
COPY ./tracker/arcadia_tracker ./tracker/arcadia_tracker

# The finished binary is placed in /usr/local/bin because it can't be conditionally copied from the runtime stage, so
# both prod and dev can be copied from the same place.
RUN --mount=type=cache,id=arcadia-tracker-target,target=/app/target,sharing=locked \
    --mount=type=cache,id=arcadia-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    if [ "$PRODUCTION_BUILD" = "true" ]; then \
    SQLX_OFFLINE=true cargo build --bin arcadia_tracker --release && mv /app/target/release/arcadia_tracker /usr/local/bin; \
    else \
    SQLX_OFFLINE=true cargo build --bin arcadia_tracker && mv /app/target/debug/arcadia_tracker /usr/local/bin; \
    fi

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y libssl-dev openssl curl pkg-config ca-certificates

COPY --from=builder /usr/local/bin/arcadia_tracker /usr/local/bin

ENTRYPOINT ["/usr/local/bin/arcadia_tracker"]
