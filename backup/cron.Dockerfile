# The backup_cron image. Based on postgres:18-alpine so pg_dump matches the db service exactly;
# restic, the mariadb client and bash are added, tzdata for backup.cron_timezone. No docker cli:
# the job dumps over the compose network and runs restic in this container. The repository is
# mounted at /arcadia by compose.

# Build stage: compile the OTLP reporter to a static binary. Build context is the repo root.
FROM rust:1-alpine AS otlp
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY backup/otlp/ ./
RUN cargo build --release

FROM postgres:18-alpine
RUN apk add --no-cache bash restic mariadb-client tzdata
COPY --from=otlp /src/target/release/arcadia-otlp /usr/local/bin/arcadia-otlp
ENTRYPOINT ["/arcadia/backup/cron.sh"]
ENV ARCADIA_BACKUP_CONTAINER=1
