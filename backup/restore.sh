#!/usr/bin/env bash
# Restores a snapshot made by backup.sh: configuration files, volumes (docker) or data directories
# (host) and databases. Usage: backup/restore.sh [snapshot] [-y], the snapshot defaults to latest.
# Configured by the `backup` section of config.yml, see docs/src/backup.md.
cd "$(dirname "$0")/.." || exit 1
. backup/lib.sh
# Host mode drops and creates the database as the postgres user (runuser) and restic restores the
# data directories with their owners. Docker mode only needs access to docker.
if [ "$MODE" = host ] && [ "$(id -u)" != 0 ]; then echo "run restore.sh as root in host mode" >&2; exit 1; fi

SNAP=latest
YES=
RESET=
SERVICES=
for arg in "$@"; do
    case $arg in -y) YES=1 ;; *) SNAP=$arg ;; esac
done

# The data about to be overwritten, so the confirmation is not blind. Best effort: the services are
# still running at this point (stop_services comes later), and a query that fails prints a note
# rather than aborting. Docker reaches the databases through the compose network via exec; host mode
# connects directly, same as restore does below.
mariadb_state() {
    local svc=$1 db=$2
    echo "--- MariaDB $svc ---"
    if [ "$MODE" = docker ]; then
        db=$(docker compose exec -T "$svc" printenv MYSQL_DATABASE 2> /dev/null) || { echo "(unavailable)"; return 0; }
    fi
    [ -n "$db" ] || { echo "(no database configured)"; return 0; }
    mariadb_cli "$svc" -qAt "$db" -e \
        "SELECT CONCAT(ROUND(SUM(data_length+index_length)/1024/1024,2),' MB, ',COUNT(*),' tables, ~',COALESCE(SUM(table_rows),0),' rows') FROM information_schema.tables WHERE table_schema='$db';" \
        2> /dev/null || echo "(unavailable)"
}

check_current_state() {
    local user name
    user=$(config_value database user)
    name=$(config_value database name)
    echo "=== Current data (will be overwritten) ==="
    echo "--- Postgres ($name) ---"
    local q="SELECT pg_size_pretty(pg_database_size('$name')), (SELECT count(*) FROM information_schema.tables WHERE table_schema='public'), coalesce((SELECT sum(n_live_tup) FROM pg_stat_user_tables),0)::bigint;"
    if [ "$MODE" = docker ]; then
        docker compose exec -T db psql -qAt -U "$user" -d "$name" -c "$q" 2> /dev/null || echo "(unavailable)"
    else
        PGPASSWORD=$(config_value database password) psql -qAt \
            -h "$(config_value database host)" -p "$(config_value database port)" \
            -U "$user" -d "$name" -c "$q" 2> /dev/null || echo "(unavailable)"
    fi
    mariadb_state ergo_database "$(cfg ergo_db)"
    mariadb_state chevereto_database "$(cfg chevereto_db)"
    echo "=========================================="
}

confirm() {
    restic snapshots "$SNAP"
    check_current_state
    if [ -n "$YES" ]; then return 0; fi
    local answer
    read -rp "Overwrite the databases, data and configuration files with snapshot $SNAP? Type yes: " answer
    [ "$answer" = yes ] || exit 1
    if [ "$MODE" = docker ]; then
        read -rp "Also recreate the database volumes ($DB_VOLUMES), so they take the passwords of .env? [y/N] " answer
        case $answer in y | Y | yes) RESET=1 ;; esac
    fi
}

# Docker: postgres and mariadb apply the passwords of .env only when their volume is first created, so
# a volume created before .env was edited refuses them (mariadb root during the restore, the backend
# afterwards). The dumps hold the databases only, so nothing else is lost. Asked in confirm, skipped
# with -y.
DB_VOLUMES="db_data ergo_database chevereto_database"
reset_db_volumes() {
    [ -n "$RESET" ] || return 0
    local v
    # by volume rather than by service: the mariadb services may sit in a profile that is not enabled
    for v in $DB_VOLUMES; do
        v=${COMPOSE_PROJECT_NAME}_$v
        docker ps -aq --filter volume="$v" | xargs -r docker rm -f
        docker volume rm -f "$v"
    done
}

# Docker: stops every running service, profiles included, to start them again at the end (redis and
# ergo would overwrite restored files on shutdown). Host: the services are not ours to manage.
stop_services() {
    if [ "$MODE" = docker ]; then
        SERVICES=$(docker compose ps --services --status running)
        if [ -n "$SERVICES" ]; then docker compose stop $SERVICES; fi
    else
        local host port
        host=$(config_value api host)
        port=$(config_value api port)
        if curl -sf -o /dev/null "http://${host:-127.0.0.1}:${port:-8080}/health"; then
            echo "the backend answers: stop arcadia, ergo and redis before restoring" >&2
            exit 1
        fi
    fi
}

# Host mode drops and creates the cluster as the postgres superuser, which the database section
# holds no password for: it goes through the unix socket, which only reaches the local cluster. The
# configured port picks the socket, so dropdb, createdb and psql all land on the same cluster.
# Checked before anything is overwritten.
check_postgres() {
    if [ "$MODE" = docker ]; then return 0; fi
    local host
    host=$(config_value database host)
    case $host in
        "" | localhost | 127.0.0.1) ;;
        *) echo "host mode restores the local postgres cluster, but database.host is '$host'" >&2; exit 1 ;;
    esac
}

# The dumps and configuration files. restic runs as root and restores them root owned: the files it
# overwrites keep the owner they had, the new ones get the owner of the repository, so the checkout
# stays usable without sudo. The database credentials are read from the restored config.yml from
# here on.
restore_files() {
    local f dest owner
    # restic (in docker mode, run from its image) writes to DUMP_DIR, which lands on the host at
    # DUMP_DIR_HOST through the BACKUP_DIR bind mount; the host-side reads below use DUMP_DIR_HOST.
    # SELinux labels are skipped (here and in restore_data): the snapshot carries the old host's, a
    # container may not set them, and restic exits non zero on that.
    restic restore "$SNAP:$DUMP_DIR" --target "$DUMP_DIR" --exclude-xattr security.selinux
    # The restic container restores them root owned, the mariadb dumps 0600 (mktemp): hand them to
    # the user running this script, who reads and cleans them up from the host.
    if [ "$MODE" = docker ]; then
        docker run --rm -v "$BACKUP_DIR:$BACKUP_MOUNT" --entrypoint chown "$RESTIC_IMAGE" \
            -R "$(id -u):$(id -g)" "$DUMP_DIR"
    fi
    # staged with --parents, so ergo/ and kiwiirc/ are directories of configuration files: they are
    # restored file by file, the checkout holds more in them than the snapshot (the examples), and
    # find leaves the loop empty when the snapshot holds no configuration file at all.
    while IFS= read -r f; do
        dest=${f#"$DUMP_DIR_HOST/config/"}
        owner=$(stat -c %u:%g "$dest" 2> /dev/null || stat -c %u:%g .)
        mkdir -p "$(dirname "$dest")"
        cp -a "$f" "$dest"
        chown "$owner" "$dest"
    done < <(find "$DUMP_DIR_HOST/config" -type f 2> /dev/null)
}

restore_data() {
    local t
    for t in $(data_targets); do restic restore "$SNAP:$t" --target "$t" --delete --exclude-xattr security.selinux; done
}

restore_postgres() {
    local user name port
    user=$(config_value database user)
    name=$(config_value database name)
    if [ "$MODE" = docker ]; then
        docker compose up -d --wait db
        docker compose exec -T db dropdb -U "$user" --if-exists --force "$name"
        docker compose exec -T db createdb -U "$user" "$name"
        docker compose exec -T db psql -q -v ON_ERROR_STOP=1 -U "$user" -d "$name" < "$DUMP_DIR_HOST/arcadia.sql"
    else
        port=$(config_value database port)
        runuser -u postgres -- dropdb -p "$port" --if-exists --force "$name"
        runuser -u postgres -- createdb -p "$port" -O "$user" "$name"
        PGPASSWORD=$(config_value database password) psql -q -v ON_ERROR_STOP=1 \
            -h "$(config_value database host)" -p "$port" -U "$user" -d "$name" \
            < "$DUMP_DIR_HOST/arcadia.sql"
    fi
}

# mariadb as root, in compose service $1 (docker) or on the host
mariadb_cli() {
    local service=$1
    shift
    if [ "$MODE" = docker ]; then
        docker compose exec -T "$service" sh -c 'MYSQL_PWD="$MYSQL_ROOT_PASSWORD" exec mariadb -uroot "$@"' - "$@"
    else
        mariadb "$@"
    fi
}

# $1: compose service, also the dump name. $2: database name in host mode. Skipped without a dump.
restore_mariadb() {
    [ -f "$DUMP_DIR_HOST/$1.sql" ] || return 0
    local db=$2
    if [ "$MODE" = docker ]; then
        docker compose up -d --wait "$1"
        db=$(docker compose exec -T "$1" printenv MYSQL_DATABASE)
    fi
    mariadb_cli "$1" -e "DROP DATABASE IF EXISTS \`$db\`; CREATE DATABASE \`$db\`"
    mariadb_cli "$1" "$db" < "$DUMP_DIR_HOST/$1.sql"
}

mkdir -p "$DUMP_DIR_HOST"
confirm
clean_dumps
trap clean_dumps EXIT
stop_services
reset_db_volumes
check_postgres
restore_files
restore_data
restore_postgres
restore_mariadb ergo_database "$(cfg ergo_db)"
restore_mariadb chevereto_database "$(cfg chevereto_db)"
if [ -n "$SERVICES" ]; then docker compose up -d $SERVICES; fi
