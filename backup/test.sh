#!/usr/bin/env bash
# End to end test of backup.sh, pull.sh and restore.sh: one machine plays arcadia, the backup host
# and a fresh host. Usage: backup/test.sh docker|host
#
# Works on a copy of the repository, under its own compose project and database name, so it does
# not touch a development setup. Needs sudo, restic and `ssh root@localhost` accepting the current
# user's key (the pull goes through sftp). Host mode also needs postgres and mariadb running locally,
# see .github/workflows/backup.yml.
set -euo pipefail

MODE=$1
SRC=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d)
W=$T/arcadia
export COMPOSE_PROJECT_NAME=arcadia_backup_test
DB=arcadia_backup_test
# Docker mode runs backup.sh in the backup_cron container: the host BACKUP_DIR (set in .env) is
# bind-mounted at the fixed /var/backups/arcadia and the volumes at /data/<name>. The repo and dumps
# live under BACKUP_DIR, so the test points at different repos by moving BACKUP_DIR (set_backup_dir),
# not by configuring a repo path. The B* vars are the host paths the test checks directly; CDUMPS is the
# in-container dumps path restic records in the snapshots and CREMOTES the remote_repo list config holds
# (both equal to the host paths in host mode).
if [ "$MODE" = docker ]; then
    export BACKUP_DIR="$T/backup" RESTIC_PASSWORD_FILE="$T/password"
    mkdir -p "$BACKUP_DIR"
    BREPO="$T/backup/repo"; BDUMPS="$T/backup/dumps"
    BREMOTE="$T/backup/remote"; BREMOTE2="$T/backup/remote2"
    BPULLED="$T/pulled/repo"; BPASS="$T/password"
    VOLUMES="redis_data ergo_data chevereto_storage elysia_storage"
    CDUMPS=/var/backups/arcadia/dumps
    CREMOTES="/var/backups/arcadia/remote /var/backups/arcadia/remote2"
else
    BREPO="$T/repo"; BDUMPS="$T/dumps"; BREMOTE="$T/remote"; BREMOTE2="$T/remote2"
    BPULLED="$T/pulled"; BPASS="$T/password"
    VOLUMES="${COMPOSE_PROJECT_NAME}_chevereto_storage ${COMPOSE_PROJECT_NAME}_ergo_data ${COMPOSE_PROJECT_NAME}_redis_data"
    CDUMPS="$BDUMPS"; CREMOTES="$BREMOTE $BREMOTE2"
fi
PATHS="$T/data/images $T/data/ergo"
# Docker mode: point the backup at a host directory by rewriting .env (compose and restore.sh both read
# it). The repo is always <dir>/repo and the dumps <dir>/dumps inside it.
set_backup_dir() { export BACKUP_DIR="$1"; sed -i "s|^BACKUP_DIR=.*|BACKUP_DIR=$1|" "$W/.env"; }
# random files in the directory given as $0
FILL='mkdir -p "$0/sub" && head -c 4194304 /dev/urandom > "$0/blob" && date +%N > "$0/sub/file"'

fail() { echo "FAIL: $*" >&2; exit 1; }
as_root() { sudo env COMPOSE_PROJECT_NAME="$COMPOSE_PROJECT_NAME" "$@"; }
dc() { docker compose "$@"; }
r() { sudo env RESTIC_PASSWORD_FILE="$T/password" restic -r "$@"; }
count() { r "$1" list snapshots | wc -l; }

run_backup() {
    if [ "$MODE" = docker ]; then
        dc --profile backup run --rm --entrypoint /arcadia/backup/backup.sh backup_cron
    else
        as_root backup/backup.sh
    fi
}

backup_config() { # $1: repo/dump_dir/password_file (host mode only; ignored in docker). $2: cron schedule of the backup_cron service, empty = none
    cat <<EOF
backup:
  mode: $MODE
  cron: "$2"
  cron_timezone: Europe/Berlin
EOF
    # docker mode derives these from BACKUP_DIR / RESTIC_PASSWORD_FILE (.env); only host mode sets them.
    if [ "$MODE" != docker ]; then
        cat <<EOF
  repo: $1
  password_file: $BPASS
  dump_dir: $BDUMPS
EOF
    fi
    cat <<EOF
  keep: --keep-last 20
  remote_repo: $CREMOTES
  volumes: $VOLUMES
  paths: $PATHS
  ergo_db: ergo_history
  chevereto_db: chevereto
  elysia_db: elysia
EOF
}

pull_config() { # $1: trigger
    cat <<EOF
backup_pull:
  ssh: root@localhost
  source_repo: $BREPO
  trigger: $1
  repo: $BPULLED
  password_file: $T/password
  keep: --keep-last 20
EOF
}

setup() {
    mkdir -p "$W" "$BDUMPS" "$(dirname "$BPULLED")"
    git -C "$SRC" ls-files -co --exclude-standard -z | (cd "$SRC" && xargs -0 cp --parents -t "$W")
    cd "$W"
    # compose reads the ARCADIA_<section>__<key> database credentials from .env (it only auto-loads
    # .env, not example.env). restore.sh runs on the host via sudo (no exported env), so the docker-mode
    # backup location comes from .env too: point its BACKUP_DIR / RESTIC_PASSWORD_FILE at the test paths.
    cp example.env .env
    if [ "$MODE" = docker ]; then
        sed -i -e "s|^BACKUP_DIR=.*|BACKUP_DIR=$BACKUP_DIR|" \
            -e "s|^RESTIC_PASSWORD_FILE=.*|RESTIC_PASSWORD_FILE=$RESTIC_PASSWORD_FILE|" .env
        echo "ARCADIA_DATABASE__NAME=$DB" >> .env
    fi
    echo "test-$RANDOM$RANDOM" > "$T/password"
    local db_host=localhost
    if [ "$MODE" = docker ]; then db_host=db; fi
    { sed -e "s/host: arcadiadb/host: $db_host/" -e "s/name: arcadia\$/name: $DB/" config.ci.yml
      backup_config "$BREPO" ""; } > config.yml
    echo "ergo $RANDOM" > ergo/ergo-conf.yaml
    echo "motd $RANDOM" > ergo/ergo.motd
    echo '{"test": true}' > kiwiirc/config.json
    pull_config '""' > "$T/pull.yml"
    local trigger="cd $W && COMPOSE_PROJECT_NAME=$COMPOSE_PROJECT_NAME backup/backup.sh"
    if [ "$MODE" = docker ]; then
        trigger="cd $W && COMPOSE_PROJECT_NAME=$COMPOSE_PROJECT_NAME BACKUP_DIR=$BACKUP_DIR RESTIC_PASSWORD_FILE=$BPASS docker compose --profile backup run --rm --entrypoint /arcadia/backup/backup.sh backup_cron"
    fi
    pull_config "$trigger" > "$T/pull-trigger.yml"
}

pg() { # psql as arcadia on the test database
    if [ "$MODE" = docker ]; then dc exec -T db psql -q -v ON_ERROR_STOP=1 -U arcadia -d "$DB" "$@"
    else PGPASSWORD=password psql -q -v ON_ERROR_STOP=1 -h localhost -U arcadia -d "$DB" "$@"; fi
}

my() { # $1: compose service, $2: mariadb or mariadb-dump, run as root on the service's database
    local s=$1; shift
    if [ "$MODE" = docker ]; then
        dc exec -T "$s" sh -c 'p=$1; shift; exec "$p" -uroot -p"$MYSQL_ROOT_PASSWORD" "$@" "$MYSQL_DATABASE"' - "$@"
    else
        local p=$1; shift
        sudo "$p" "$@" "$(case $s in ergo_database) echo ergo_history ;; *) echo chevereto ;; esac)"
    fi
}

ely() { # psql on elysia's database, as its owner (docker) or postgres (host)
    if [ "$MODE" = docker ]; then dc exec -T elysia_database psql -q -v ON_ERROR_STOP=1 -U elysia -d elysia "$@"
    else sudo runuser -u postgres -- psql -q -v ON_ERROR_STOP=1 -d elysia "$@"; fi
}

in_data() { # runs the shell script $1 in every volume (docker) or path (host), printing their names
    local t
    if [ "$MODE" = docker ]; then
        for t in $VOLUMES; do echo "$t"; docker run --rm -v "${COMPOSE_PROJECT_NAME}_$t:/v" alpine sh -c "$1" /v; done
    else
        for t in $PATHS; do echo "$t"; sudo mkdir -p "$t"; sudo sh -c "$1" "$t"; done
    fi
}

# Synthetic schema: circular foreign keys (like arcadia's) and a trusted extension created by the
# database owner, both of which a plain dump must restore.
seed_host() {
    sudo runuser -u postgres -- psql -q -c \
        "DO \$\$ BEGIN CREATE ROLE arcadia LOGIN PASSWORD 'password'; EXCEPTION WHEN duplicate_object THEN NULL; END \$\$"
    sudo runuser -u postgres -- createdb -O arcadia "$DB"
    pg <<'EOF'
CREATE EXTENSION unaccent;
CREATE TABLE a (id int PRIMARY KEY, b_id int);
CREATE TABLE b (id int PRIMARY KEY, a_id int NOT NULL REFERENCES a);
ALTER TABLE a ADD FOREIGN KEY (b_id) REFERENCES b;
INSERT INTO a SELECT g FROM generate_series(1, 1000) g;
INSERT INTO b SELECT g, g FROM generate_series(1, 1000) g;
UPDATE a SET b_id = id;
EOF
    sudo mariadb -e "CREATE DATABASE ergo_history; CREATE DATABASE chevereto"
    sudo runuser -u postgres -- createdb elysia
}

seed_docker() {
    dc up -d --wait db redis ergo_database chevereto_database elysia_database
    dc run --rm init_db
    pg < backend/storage/migrations/fixtures/fixtures.sql
    # redis_data is backed up raw, so it needs content: the fingerprint compares the bytes of every
    # file and a restore has to bring them back. The periodic saves are switched off, so that redis
    # cannot write dump.rdb again behind the fingerprint's back.
    local p="${ARCADIA_REDIS__PASSWORD:-password}"
    if [ -f .env ]; then p=$(grep '^ARCADIA_REDIS__PASSWORD=' .env | cut -d= -f2-); fi
    dc exec -T -e REDISCLI_AUTH="${p:-password}" redis \
        sh -c 'redis-cli CONFIG SET save "" && redis-cli SET backup_test seed && redis-cli SAVE'
}

seed() {
    "seed_$MODE"
    local s
    for s in ergo_database chevereto_database; do
        my "$s" mariadb -e "CREATE TABLE t (id INT PRIMARY KEY, v TEXT); INSERT INTO t VALUES (1, 'a'), (2, 'b')"
    done
    ely -c "CREATE TABLE t (id INT PRIMARY KEY, v TEXT); INSERT INTO t VALUES (1, 'a'), (2, 'b')"
    in_data "$FILL" > /dev/null
}

wipe() { # what a fresh host lacks: containers, volumes, databases, data directories
    if [ "$MODE" = docker ]; then
        docker ps -aq --filter "label=com.docker.compose.project=$COMPOSE_PROJECT_NAME" | xargs -r docker rm -f > /dev/null
        docker volume ls -q --filter "name=${COMPOSE_PROJECT_NAME}_" | xargs -r docker volume rm > /dev/null
    else
        sudo runuser -u postgres -- dropdb --if-exists --force "$DB"
        sudo mariadb -e "DROP DATABASE IF EXISTS ergo_history; DROP DATABASE IF EXISTS chevereto"
        sudo runuser -u postgres -- dropdb --if-exists --force elysia
        sudo rm -rf $PATHS
    fi
}

cleanup() {
    wipe > /dev/null 2>&1 || true
    if [ "$MODE" = docker ]; then
        docker network ls -q --filter "label=com.docker.compose.project=$COMPOSE_PROJECT_NAME" | xargs -r docker network rm > /dev/null
    fi
    sudo rm -rf "$T"
}
pull() { ARCADIA_CONFIG="$T/$1" backup/pull.sh; }

mutate() {
    pg -c "CREATE TABLE IF NOT EXISTS backup_test (v text); INSERT INTO backup_test VALUES ('$RANDOM')"
    my ergo_database mariadb -e "INSERT INTO t (id, v) SELECT MAX(id) + 1, 'm' FROM t"
    ely -c "INSERT INTO t (id, v) SELECT MAX(id) + 1, 'm' FROM t"
    in_data 'date +%N > "$0/sub/mutated"' > /dev/null
}

# Everything a restore must bring back. pg_dump 18 writes a random \restrict key in every dump.
fingerprint() {
    local s
    if [ "$MODE" = docker ]; then dc exec -T db pg_dump -U arcadia -d "$DB" --no-owner
    else PGPASSWORD=password pg_dump -h localhost -U arcadia -d "$DB" --no-owner; fi |
        grep -Ev '^\\(un)?restrict ' | sort
    for s in ergo_database chevereto_database; do my "$s" mariadb-dump --skip-extended-insert --skip-dump-date | sort; done
    if [ "$MODE" = docker ]; then dc exec -T elysia_database pg_dump -U elysia -d elysia
    else sudo runuser -u postgres -- pg_dump elysia; fi | grep -Ev '^\\(un)?restrict ' | sort
    in_data 'cd "$0" && find . -type f -exec sha256sum {} + | sort'
    sha256sum config.yml ergo/ergo-conf.yaml ergo/ergo.motd kiwiirc/config.json
}

trap cleanup EXIT

setup
seed

# backup.sh: snapshot with the dumps and configs, pushed to remote_repo, only its own files cleaned
echo keep > "$BDUMPS/unrelated"
run_backup
[ "$(count "$BREPO")" = 1 ] || fail "backup.sh made no snapshot"
[ "$(count "$BREMOTE")" = 1 ] || fail "backup.sh did not copy the snapshot to remote_repo"
[ "$(count "$BREMOTE2")" = 1 ] || fail "backup.sh did not copy to the second remote_repo"
[ "$(ls -A "$BDUMPS")" = unrelated ] || fail "dump_dir not cleaned, or more than the dumps removed: $(ls -A "$BDUMPS")"
r "$BREPO" ls latest > "$T/ls"
for f in arcadia.sql ergo_database.sql chevereto_database.sql elysia_database.sql config/config.yml config/ergo/ergo-conf.yaml \
    config/ergo/ergo.motd config/kiwiirc/config.json; do
    grep -qx "$CDUMPS/$f" "$T/ls" || fail "$f missing from the snapshot"
done
! grep -qx "$CDUMPS/unrelated" "$T/ls" || fail "unrelated file stored in the snapshot"

# unchanged data is not stored again
size=$(sudo du -sb "$BREPO" | cut -f1)
run_backup
added=$(( $(sudo du -sb "$BREPO" | cut -f1) - size ))
[ "$added" -lt 2097152 ] || fail "a backup without changes added $added bytes"

# a stopped optional service is skipped, not dumped empty
if [ "$MODE" = docker ]; then
    dc stop chevereto_database
    run_backup
    r "$BREPO" ls latest > "$T/ls"
    if grep -q chevereto_database.sql "$T/ls"; then fail "dumped the stopped chevereto_database"; fi
    dc up -d --wait chevereto_database
fi

# pull.sh: copies every missing snapshot, fails when there is none
pull pull.yml || fail "pull.sh failed"
[ "$(count "$BPULLED")" = "$(count "$BREPO")" ] || fail "pull.sh did not copy every snapshot"
if pull pull.yml; then fail "pull.sh succeeded without a new snapshot"; fi

# catch up: one backup made by arcadia's own schedule, one by the trigger, both copied by one pull
mutate
fingerprint > "$T/before"
n=$(count "$BPULLED")
run_backup
pull pull-trigger.yml || fail "pull.sh with a trigger failed"
[ "$(count "$BPULLED")" = $((n + 2)) ] || fail "pull.sh did not catch up on both snapshots"

# restore.sh asks first
if echo no | as_root backup/restore.sh; then fail "restore.sh ran without confirmation"; fi

# same host, services running: changed since the backup, then stopped, restored, started again
mutate
running=""
if [ "$MODE" = docker ]; then running=$(dc ps --services --status running | sort); fi
as_root backup/restore.sh -y
fingerprint | diff "$T/before" - || fail "restoring on the same host changed the data"
if [ "$MODE" = docker ]; then
    [ "$(dc ps --services --status running | sort)" = "$running" ] || fail "restore.sh did not start the services again"
fi

# fresh host: nothing but a checkout, the password and the backup host's repository. In docker mode the
# pulled repo is $T/pulled/repo, so point BACKUP_DIR at $T/pulled for the restore.
wipe
sudo rm -rf "$BREPO" "$BDUMPS"
rm config.yml ergo/ergo-conf.yaml ergo/ergo.motd kiwiirc/config.json
if [ "$MODE" = docker ]; then set_backup_dir "$(dirname "$BPULLED")"; fi
backup_config "$BPULLED" "" > config.yml
as_root backup/restore.sh latest -y
fingerprint | diff "$T/before" - || fail "restoring on a fresh host from the pulled repository differs"

# backup_cron: turns backup.cron into a crontab and runs backup.sh from it, in a container started
# with the mounts of the backup section. Waits for the minute the schedule fires in.
if [ "$MODE" = docker ]; then
    # back to a fresh BACKUP_DIR (the fresh-host restore moved it); the fresh-host phase removed the repo,
    # so the scheduled run starts the repo from empty and a non-zero snapshot count proves it ran.
    set_backup_dir "$T/backup"
    mkdir -p "$BDUMPS"
    { sed -e "s/host: arcadiadb/host: db/" -e "s/name: arcadia\$/name: $DB/" config.ci.yml
      backup_config "$BREPO" '* * * * *'; } > config.yml
    dc up -d --wait db ergo_database
    dc --profile backup up -d --wait backup_cron
    crontab=""
    for i in $(seq 5); do crontab=$(dc exec -T backup_cron cat /etc/crontabs/root 2> /dev/null) && break; sleep 1; done
    case $crontab in
        *"/arcadia/backup/backup.sh"*) ;;
        *) fail "backup_cron scheduled nothing usable: ${crontab:-<no crontab>}" ;;
    esac
    for i in $(seq 75); do [ "$(count "$BREPO" 2> /dev/null)" != 0 ] && break; sleep 2; done
    [ "$(count "$BREPO" 2> /dev/null)" != 0 ] || fail "the scheduled backup never ran"
    r "$BREPO" ls latest > "$T/ls"
    # the dump proves the job reached the database over the network, the volume file that it
    # backed a volume up raw, redis_data included
    grep -qx "$CDUMPS/arcadia.sql" "$T/ls" || fail "the scheduled backup dumped no database"
    grep -qx "$CDUMPS/ergo_database.sql" "$T/ls" || fail "the scheduled backup dumped no mariadb database"
    grep -qx "/data/redis_data/dump.rdb" "$T/ls" ||
        fail "the scheduled backup stored no redis volume"
    dc --profile backup rm -sf backup_cron
fi

echo "backup test passed ($MODE)"
