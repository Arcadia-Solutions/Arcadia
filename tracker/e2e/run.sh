#!/usr/bin/env bash
# End to end tests of the tracker with real torrent clients, one test per supported client version.
# usage: ./run.sh [test name...]   (all tests when none is given)
set -euo pipefail
cd "$(dirname "$0")"

FIXTURES=../arcadia_tracker/tests/fixtures
declare -A PASSKEYS=(
    [seeder]=5eede75eede75eede75eede75eede75e
    [leecher]=1eec4e121eec4e121eec4e121eec4e12
    [leecher2]=2eec4e122eec4e122eec4e122eec4e12
)
SIZE=$((16 * 1024 * 1024))
# Pieces in flight are discarded when a client restarts and get transferred again. More than
# this means some bytes were counted twice.
MAX_TRANSFERRED=$((SIZE + SIZE / 4))
# Tracker configuration overrides for the scenarios about peers that stop announcing. Their
# inactive_peer_ttl stays at its default of several minutes.
SHORT_ACTIVE_PEER_TTL=(ARCADIA_TRACKER__ACTIVE_PEER_TTL=10 ARCADIA_TRACKER__PEER_EXPIRY_INTERVAL=1)

export HOST_UID HOST_GID WORK_DIR CLIENT_IMAGE
# only ever holds a directory created by this script, which setup and teardown delete
WORK_DIR=""
HOST_UID=$(id -u)
HOST_GID=$(id -g)

compose() { docker compose "$@"; }
sql() { compose exec -T db psql -U arcadia -d arcadia -tA -c "$1"; }
fail() {
    echo "FAIL: $*" >&2
    exit 1
}

# Polls a query until it returns the expected value, running an optional command before each try
# usage: wait_sql <description> <query> <expected> [command...]
wait_sql() {
    local description=$1 query=$2 expected=$3 actual=""
    shift 3
    for _ in $(seq 60); do
        [ $# -gt 0 ] && "$@"
        actual=$(sql "$query")
        [ "$actual" = "$expected" ] && return 0
        sleep 1
    done
    fail "$description: expected '$expected', got '$actual'"
}

assert_eq() { [ "$2" = "$3" ] || fail "$1: expected '$3', got '$2'"; }
assert_between() { [ "$2" -ge "$3" ] && [ "$2" -le "$4" ] || fail "$1: expected $3..$4, got $2"; }

user_id() { sql "SELECT id FROM users WHERE username = 'e2e_$1'"; }
user_value() { sql "SELECT $2 FROM users WHERE username = 'e2e_$1'"; }
container_ip() { docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$(compose ps -q "$1")"; }
torrent_counts_query="SELECT seeders || '/' || leechers || '/' || times_completed FROM torrents WHERE id = 1"

# Starts the db and the tracker, with a fresh database and payload
# usage: setup [tracker configuration override...]
setup() {
    compose down -v --remove-orphans >/dev/null 2>&1 || true
    [ -n "${WORK_DIR:-}" ] && rm -rf "$WORK_DIR"
    WORK_DIR=$(mktemp -d)
    mkdir -p "$WORK_DIR"/{seeder,leecher,leecher2}/{config,downloads}
    printf '%s\n' "$@" >"$WORK_DIR/tracker.env"
    head -c "$SIZE" /dev/urandom >"$WORK_DIR/seeder/downloads/payload.bin"
    local role
    for role in "${!PASSKEYS[@]}"; do
        INFO_HASH=$(python3 -I make_torrent.py "$WORK_DIR/seeder/downloads/payload.bin" \
            "http://tracker:8081/announce/${PASSKEYS[$role]}" "$WORK_DIR/$role.torrent")
    done

    compose up -d --wait db
    compose run --rm init_db >/dev/null
    cat "$FIXTURES/with_test_title_group.sql" "$FIXTURES/with_test_edition_group.sql" \
        "$FIXTURES/with_test_torrent.sql" seed.sql |
        compose exec -T db psql -q -v ON_ERROR_STOP=1 -v info_hash="$INFO_HASH" -v size="$SIZE" -U arcadia -d arcadia
    compose up -d --wait --build tracker
}

teardown() {
    local status=$?
    if [ $status -ne 0 ]; then
        compose logs tracker seeder leecher leecher2 >&2
    fi
    compose down -v --remove-orphans >/dev/null 2>&1 || true
    [ -n "${WORK_DIR:-}" ] && rm -rf "$WORK_DIR"
}
trap teardown EXIT

# What the tracker recorded for a user has to match what its client announced, whatever the
# client did: every peer session (a new peer_id per client restart) is counted once, from the last
# amounts the client announced in it. Clients do not have to agree with each other: data in
# flight when a leecher stops is counted as uploaded by the seeder only.
assert_user_consistent() {
    local role=$1 id up down
    id=$(user_id "$role")
    up=$(user_value "$role" real_uploaded)
    down=$(user_value "$role" real_downloaded)
    echo "    $role: uploaded $up, downloaded $down, size $SIZE"
    assert_eq "$role uploaded = its sessions" "$(sql "SELECT coalesce(sum(uploaded), 0) FROM peers WHERE user_id = $id")" "$up"
    assert_eq "$role downloaded = its sessions" "$(sql "SELECT coalesce(sum(downloaded), 0) FROM peers WHERE user_id = $id")" "$down"
    # upload_factor 200, download_factor 50, rounded down on every announce
    assert_eq "$role credited uploaded" "$(user_value "$role" uploaded)" $((up * 2))
    assert_between "$role credited downloaded" "$(user_value "$role" downloaded)" $((down / 2 - 10)) $((down / 2))
    assert_eq "$role activity" \
        "$(sql "SELECT real_uploaded || '/' || real_downloaded FROM torrent_activities WHERE torrent_id = 1 AND user_id = $id")" \
        "$up/$down"
}

# Once all leechers completed: every client reported all its traffic, each has a single active
# seeding peer and its user is consistent
# usage: assert_all_reported <agent prefix> <expected torrent counts> <role...>
assert_all_reported() {
    local agent_prefix=$1 counts=$2 role
    shift 2
    local leechers=$(($# - 1))
    # uploads are only reported on the next announce, and clients may skip a forced reannounce
    # that comes too early
    wait_sql "all uploads reported" "SELECT sum(real_uploaded) >= $leechers * $SIZE FROM users" t \
        client_reannounce "$@"
    wait_sql "torrent seeders/leechers/times_completed" "$torrent_counts_query" "$counts"
    for role in "$@"; do
        assert_user_consistent "$role"
        assert_eq "$role active peer" \
            "$(sql "SELECT host(p.ip), p.port, p.seeder, p.left, starts_with(p.agent, '$agent_prefix')
                    FROM peers p JOIN users u ON u.id = p.user_id WHERE u.username = 'e2e_$role' AND p.active")" \
            "$(container_ip "$role")|6881|t|0|t"
    done
    assert_eq "no announce errors" "$(sql "SELECT count(*) FROM announce_errors")" 0
}

assert_seeder_leecher_bounds() {
    assert_between "seeder uploaded" "$(user_value seeder real_uploaded)" "$SIZE" "$MAX_TRANSFERRED"
    assert_between "leecher downloaded" "$(user_value leecher real_downloaded)" "$SIZE" "$MAX_TRANSFERRED"
    assert_eq "leecher nothing uploaded" "$(user_value leecher real_uploaded)" 0
    assert_eq "seeder nothing downloaded" "$(user_value seeder real_downloaded)" 0
}

assert_payload() {
    cmp -s "$WORK_DIR/seeder/downloads/payload.bin" "$WORK_DIR/$1/downloads/payload.bin" ||
        fail "payload downloaded by $1 differs"
}

### qBittorrent (official qbittorrent-nox image)

declare -A QBT_PORTS=([seeder]=18080 [leecher]=18081 [leecher2]=18082)

qbt() {
    local role=$1 endpoint=$2
    shift 2
    curl -sf "http://127.0.0.1:${QBT_PORTS[$role]}/api/v2/$endpoint" "$@"
}

qbt_wait_ready() {
    for _ in $(seq 60); do
        qbt "$1" app/version >/dev/null 2>&1 && return 0
        sleep 1
    done
    fail "$1 web API not reachable"
}

qbt_progress() { { qbt "$1" "torrents/info?hashes=$INFO_HASH" || echo '[]'; } | python3 -c 'import json,sys; t=json.load(sys.stdin); print(t[0]["progress"] if t else -1)'; }

# Waits until the torrent progress of a client reaches the given fraction
qbt_wait_progress() {
    local role=$1 min=$2
    for _ in $(seq 240); do
        python3 -c "import sys; sys.exit(not $(qbt_progress "$role") >= $min)" && return 0
        sleep 0.5
    done
    fail "$role did not reach progress $min, at $(qbt_progress "$role")"
}

qbt_up() {
    local role
    for role in "$@"; do
        mkdir -p "$WORK_DIR/$role/config/qBittorrent/config"
        cp qBittorrent.conf "$WORK_DIR/$role/config/qBittorrent/config/"
    done
    compose up -d "$@"
    for role in "$@"; do
        qbt_wait_ready "$role"
    done
}

qbt_add() { qbt "$1" torrents/add -F "torrents=@$WORK_DIR/$1.torrent" -F savepath=/downloads >/dev/null; }

qbt_reannounce() {
    local role
    for role in "$@"; do
        qbt "$role" torrents/reannounce -d "hashes=$INFO_HASH"
    done
}

# Fresh stack with the given clients up and the seeder seeding
# usage: qbt_start <role...> [-- tracker configuration override...]
qbt_start() {
    local roles=()
    while [ $# -gt 0 ] && [ "$1" != -- ]; do
        roles+=("$1")
        shift
    done
    [ $# -gt 0 ] && shift
    setup "$@"
    qbt_up "${roles[@]}"
    qbt_add seeder
    qbt_wait_progress seeder 1
    # Otherwise the leecher may announce first, get no peer, and only retry after the interval
    wait_sql "seeder peer active" \
        "SELECT p.active FROM peers p JOIN users u ON u.id = p.user_id WHERE u.username = 'e2e_seeder' AND p.seeder" t
}

# Starts the leecher slowly, and waits until it got part of the payload
qbt_leecher_partially_downloaded() {
    qbt leecher transfer/setDownloadLimit -d limit=$((2 * 1024 * 1024))
    qbt_add leecher
    qbt_wait_progress leecher 0.3
}

qbt_leecher_finish() {
    qbt leecher transfer/setDownloadLimit -d limit=0
    qbt_wait_progress leecher 1
    assert_payload leecher
}

test_qbittorrent() {
    local version=$1 stop_endpoint=torrents/stop
    # qBittorrent 4.x calls stopping a torrent pausing it
    [[ $version == 4.* ]] && stop_endpoint=torrents/pause
    CLIENT_IMAGE=qbittorrentofficial/qbittorrent-nox:$version
    client_reannounce() { qbt_reannounce "$@"; }

    echo "=== qBittorrent $version: transfer then stop"
    qbt_start seeder leecher
    qbt_add leecher
    qbt_wait_progress leecher 1
    assert_payload leecher
    assert_all_reported "qBittorrent/" 2/0/1 seeder leecher
    assert_seeder_leecher_bounds
    qbt leecher "$stop_endpoint" -d "hashes=$INFO_HASH"
    wait_sql "leecher peer inactive after stop" \
        "SELECT p.active FROM peers p JOIN users u ON u.id = p.user_id WHERE u.username = 'e2e_leecher'" "f"
    wait_sql "seeders after stop" "SELECT seeders || '/' || leechers FROM torrents WHERE id = 1" "1/0"

    echo "=== qBittorrent $version: leecher restarted mid-download"
    qbt_start seeder leecher
    qbt_leecher_partially_downloaded
    compose restart -t 30 leecher
    qbt_wait_ready leecher
    qbt_leecher_finish
    assert_all_reported "qBittorrent/" 2/0/1 seeder leecher
    assert_seeder_leecher_bounds

    echo "=== qBittorrent $version: swarm of one seeder and two leechers"
    qbt_start seeder leecher leecher2
    # slow seeder, so that the leechers get pieces from each other
    qbt seeder transfer/setUploadLimit -d limit=$((1024 * 1024))
    qbt_add leecher
    qbt_add leecher2
    qbt_wait_progress leecher 1
    qbt_wait_progress leecher2 1
    assert_payload leecher
    assert_payload leecher2
    assert_all_reported "qBittorrent/" 3/0/2 seeder leecher leecher2
    local role
    for role in leecher leecher2; do
        assert_between "$role downloaded" "$(user_value $role real_downloaded)" "$SIZE" "$MAX_TRANSFERRED"
    done
    [ $(($(user_value leecher real_uploaded) + $(user_value leecher2 real_uploaded))) -gt 0 ] ||
        fail "the leechers did not upload to each other"

    echo "=== qBittorrent $version: leecher killed mid-download"
    qbt_start seeder leecher -- "${SHORT_ACTIVE_PEER_TTL[@]}"
    qbt_leecher_partially_downloaded
    compose kill leecher
    # the seeder keeps announcing, the leecher gets marked as inactive
    wait_sql "killed leecher reaped" "$torrent_counts_query" 1/0/0 qbt_reannounce seeder
    # a new container, like a reboot after the crash: restarting the killed one reuses its pid,
    # so qBittorrent sees its own stale lock file as another running instance and exits
    compose rm -f leecher
    compose up -d leecher
    qbt_wait_ready leecher
    qbt_leecher_finish
    # What the killed session downloaded was never announced, and the client may have lost it
    # too, without resume data saved before the crash, so the seeder may have sent it twice
    assert_all_reported "qBittorrent/" 2/0/1 seeder leecher
    assert_between "seeder uploaded" "$(user_value seeder real_uploaded)" "$SIZE" $((SIZE + MAX_TRANSFERRED))

    echo "=== qBittorrent $version: leecher frozen mid-download, then back"
    qbt_start seeder leecher -- "${SHORT_ACTIVE_PEER_TTL[@]}"
    qbt_leecher_partially_downloaded
    # like a suspended laptop: no announce and no stopped event, then the same session resumes
    compose pause leecher
    wait_sql "frozen leecher marked inactive" "$torrent_counts_query" 1/0/0 qbt_reannounce seeder
    # a few more expiry passes, still far from inactive_peer_ttl
    for _ in $(seq 5); do
        qbt_reannounce seeder
        sleep 1
    done
    compose unpause leecher
    qbt_leecher_finish
    assert_all_reported "qBittorrent/" 2/0/1 seeder leecher
    assert_seeder_leecher_bounds

    echo "=== qBittorrent $version: client not in the whitelist"
    setup 'ARCADIA_TRACKER__ALLOWED_TORRENT_CLIENTS=[XX]'
    qbt_up leecher
    qbt_add leecher
    wait_sql "whitelist error recorded" \
        "SELECT error_code FROM announce_errors WHERE user_id = $(user_id leecher)" torrent_client_not_in_whitelist
    for _ in $(seq 30); do
        qbt leecher "torrents/trackers?hash=$INFO_HASH" | grep -q '"msg":"torrent client not in whitelist"' && break
        sleep 1
    done
    qbt leecher "torrents/trackers?hash=$INFO_HASH" | grep -q '"msg":"torrent client not in whitelist"' ||
        fail "client does not show the whitelist error: $(qbt leecher "torrents/trackers?hash=$INFO_HASH")"
    assert_eq "no peer for rejected client" "$(sql "SELECT count(*) FROM peers")" 0
    assert_eq "torrent counts untouched" "$(sql "$torrent_counts_query")" 0/0/0

    echo "=== qBittorrent $version: ok"
}

### Tests, one per supported client version

qbittorrent-4.6.7() { test_qbittorrent 4.6.7-1; }
qbittorrent-5.3.0() { test_qbittorrent 5.3.0-2; }

TESTS=(qbittorrent-4.6.7 qbittorrent-5.3.0)
for test in "${@:-${TESTS[@]}}"; do
    "$test"
done
