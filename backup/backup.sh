#!/usr/bin/env bash
# Dumps the databases, then snapshots the dumps, the configuration files and the volumes (docker) or
# data directories (host) into the local restic repository, prunes it and copies the snapshots to
# backup.remote_repo when set. Configured by the `backup` section of config.yml, see
# docs/src/backup.md.
cd "$(dirname "$0")/.." || exit 1
. backup/lib.sh

# The restic `backup --json` summary line of the last snapshot, reported to OTLP by report_otlp.
RESTIC_SUMMARY_JSON=""

dump_postgres() {
    local user name
    user=$(config_value database user)
    name=$(config_value database name)
    PGPASSWORD=$(config_value database password) pg_dump \
        -h "$(config_value database host)" -p "$(config_value database port)" \
        -U "$user" -d "$name" --no-owner --no-privileges > "$DUMP_DIR/arcadia.sql"
}

# $1: compose service, also the dump name. $2: database name (host mode). $3: env prefix for the
# per-database credentials (docker mode). Docker: dumped over the compose network as the owning
# user. A service that is not running (optional profile) is skipped; a reachable one whose dump
# fails or is empty fails the run, and nothing empty is ever stored: restore DROP+CREATEs from the
# dump and would wipe the database.
dump_mariadb() {
    if [ "$MODE" = docker ]; then
        local u p d tmp
        eval "u=\${${3}_USER:-}"; eval "p=\${${3}_PASSWORD:-}"; eval "d=\${${3}_NAME:-}"
        [ -n "$u" ] && [ -n "$d" ] || return 0
        # optional service not running => skip. The subshell opens and closes the probe socket.
        (exec 3<> "/dev/tcp/$1/3306") 2> /dev/null || return 0
        tmp=$(mktemp "$DUMP_DIR/.tmp.$1.XXXXXX")
        # reachable: a dump failure is a real error (bad creds, etc.), fail, do not skip silently.
        # --skip-ssl: the alpine mariadb client defaults to TLS over TCP, which the mariadb server
        # image does not offer (error 2026), so disable it on this in-network connection.
        # sed: strip the sandbox-mode preamble (`/*M!999999\- ... */`) newer mariadb-dump emits; a
        # restore client that does not support it aborts with "Unknown command '\-'". pipefail keeps
        # a mariadb-dump failure fatal through the pipe.
        if ! MYSQL_PWD="$p" mariadb-dump --skip-ssl -h "$1" -u "$u" --single-transaction "$d" \
                | sed '/enable the sandbox mode/d' > "$tmp"; then
            echo "mariadb-dump failed for $1 (reachable but dump errored)" >&2; rm -f "$tmp"; return 1
        fi
        [ -s "$tmp" ] || { echo "empty mariadb dump for $1" >&2; rm -f "$tmp"; return 1; }
        mv "$tmp" "$DUMP_DIR/$1.sql"
    elif [ -n "$2" ]; then
        # Strip the sandbox-mode preamble too, so host-made dumps restore on any client (e.g. the
        # documented docker <-> bare metal cross restore).
        mariadb-dump --single-transaction "$2" | sed '/enable the sandbox mode/d' > "$DUMP_DIR/$1.sql"
    fi
}

stage_configs() {
    local f
    mkdir -p "$DUMP_DIR/config"
    for f in $CONFIG_FILES; do
        if [ -f "$f" ]; then cp -p --parents "$f" "$DUMP_DIR/config/"; fi
    done
}

snapshot() {
    local targets raw rc
    mapfile -t targets < <(data_targets)
    restic cat config > /dev/null 2>&1 || restic init
    # --json so the run's stats (files/bytes added, duration) can be reported to OTLP. The output is
    # captured: on success keep the summary line for report_otlp, on failure print it so the container
    # logs still show restic's error.
    raw=$(mktemp)
    if restic backup --json --host arcadia --tag arcadia "$DUMP_DIR"/*.sql "$DUMP_DIR/config" "${targets[@]}" > "$raw"; then
        RESTIC_SUMMARY_JSON=$(grep '"message_type":"summary"' "$raw" | tail -1 || true)
    else
        rc=$?; cat "$raw" >&2; rm -f "$raw"; return "$rc"
    fi
    rm -f "$raw"
    # shellcheck disable=SC2046 # keep holds several flags
    restic forget --host arcadia --prune $(cfg keep)
}

# Best-effort OTLP metrics for the run: status, wall-clock duration, and restic's backup summary.
# Reports only when telemetry.otlp_endpoint is set, and never fails the backup. $1: exit code, $2: wall duration in ms.
# The configured endpoint is the backend's OTLP gRPC address (host:4317); the metrics go over OTLP/HTTP,
# so reuse the host and swap the port for 4318. Assumes the documented host:port form.
# The payload is built by backup/otlp.jq and posted with curl, both in the backup image. In host mode,
# a missing jq or curl skips telemetry and leaves the run unaffected.
report_otlp() {
    local ep
    ep=$(config_value telemetry otlp_endpoint)
    [ -n "$ep" ] && command -v jq > /dev/null && command -v curl > /dev/null || return 0
    printf '%s' "$RESTIC_SUMMARY_JSON" \
        | jq -cRn --arg code "$1" --arg ms "$2" --arg now "$(date +%s)000000000" -f backup/otlp.jq \
        | curl -sf -m 5 -o /dev/null -H 'Content-Type: application/json' --data-binary @- "${ep%:*}:4318/v1/metrics" \
        || true
}

push() {
    local remote
    for remote in $REMOTES; do
        restic -r "$remote" cat config > /dev/null 2>&1 ||
            restic -r "$remote" init --from-repo "$REPO" --copy-chunker-params
        restic -r "$remote" copy --from-repo "$REPO"
        # shellcheck disable=SC2046 # keep holds several flags
        restic -r "$remote" forget --host arcadia --prune $(cfg keep)
    done
}

mkdir -p "$DUMP_DIR" "$REPO"
umask 077
clean_dumps
_backup_started=$(date +%s)
# Clean the dumps and report the run (status, exit code, duration) on any exit. $? first: it is the
# script's exit code, which report_otlp turns into ok/err.
# shellcheck disable=SC2154 # _code is assigned at the start of the same trap, before it is read
trap '_code=$?; clean_dumps; report_otlp "$_code" "$(( ($(date +%s) - _backup_started) * 1000 ))"' EXIT
dump_postgres
dump_mariadb ergo_database "$(cfg ergo_db)" ERGO_DB
dump_mariadb chevereto_database "$(cfg chevereto_db)" CHEVERETO_DB
stage_configs
snapshot
push
