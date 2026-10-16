# The backup_cron image. Based on postgres:18-alpine so pg_dump matches the db service exactly;
# restic, the mariadb client and bash are added, tzdata for backup.cron_timezone. No docker cli:
# the job dumps over the compose network and runs restic in this container. The repository is
# mounted at /arcadia by compose.
FROM postgres:18-alpine
RUN apk add --no-cache bash restic mariadb-client tzdata curl jq
ENTRYPOINT ["/arcadia/backup/cron.sh"]
ENV ARCADIA_BACKUP_CONTAINER=1
