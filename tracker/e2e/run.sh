#!/usr/bin/env bash
# End to end tests of the tracker with real torrent clients, one test per supported client version.
# usage: ./run.sh [test name...]   (all tests when none is given)
# Tests run in parallel, JOBS of them at a time (default: the amount of CPUs), each in its own
# compose project, and their logs are kept in LOG_DIR (default: a new temporary directory).
set -euo pipefail
SELF=$(realpath "$0")
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
# Long enough for Transmission 4.0, which ignores reannounce requests for 2 minutes after the
# first announce of a torrent
# usage: wait_sql <description> <query> <expected> [command...]
wait_sql() {
    local description=$1 query=$2 expected=$3 actual=""
    shift 3
    for _ in $(seq 150); do
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

# Clients may leave files the host user cannot delete (qBittorrent 4.4 ignores PUID and gives
# them to its own user), so the content of WORK_DIR is deleted from a container
remove_work_dir() {
    [ -n "${WORK_DIR:-}" ] || return 0
    docker run --rm -v "$WORK_DIR:/work" postgres:18 find /work -mindepth 1 -delete
    rm -rf "$WORK_DIR"
}

# Starts the db and the tracker, with a fresh database and payload
# usage: setup [tracker configuration override...]
setup() {
    compose down -v --remove-orphans >/dev/null 2>&1 || true
    remove_work_dir
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
    compose up -d --wait tracker
}

teardown() {
    local status=$?
    if [ $status -ne 0 ]; then
        compose logs tracker seeder leecher leecher2 >&2
    fi
    compose down -v --remove-orphans >/dev/null 2>&1 || true
    remove_work_dir || true
}

# What the tracker recorded for a user has to match what its client announced, whatever the
# client did: every peer session (a new peer_id per client restart) is counted once, from the last
# amounts the client announced in it. Clients do not have to agree with each other: data in
# flight when a leecher stops is counted as uploaded by the seeder only.
assert_user_consistent() {
    local role=$1 id up down consistent=f
    id=$(user_id "$role")
    # Announces forced by the reannounces above may still be getting flushed, and peers, users and
    # activities are flushed in separate transactions, so this waits for a consistent snapshot.
    # The assertions after it only explain why there was none.
    for _ in $(seq 10); do
        consistent=$(sql "SELECT u.real_uploaded = (SELECT coalesce(sum(uploaded), 0) FROM peers WHERE user_id = u.id)
            AND u.real_downloaded = (SELECT coalesce(sum(downloaded), 0) FROM peers WHERE user_id = u.id)
            AND u.uploaded = u.real_uploaded * 2
            AND u.downloaded BETWEEN u.real_downloaded / 2 - 10 AND u.real_downloaded / 2
            AND (SELECT real_uploaded || '/' || real_downloaded FROM torrent_activities
                 WHERE torrent_id = 1 AND user_id = u.id) = u.real_uploaded || '/' || u.real_downloaded
            FROM users u WHERE u.id = $id")
        [ "$consistent" = t ] && break
        sleep 1
    done
    up=$(user_value "$role" real_uploaded)
    down=$(user_value "$role" real_downloaded)
    echo "    $role: uploaded $up, downloaded $down, size $SIZE"
    [ "$consistent" = t ] && return 0
    assert_eq "$role uploaded = its sessions" "$(sql "SELECT coalesce(sum(uploaded), 0) FROM peers WHERE user_id = $id")" "$up"
    assert_eq "$role downloaded = its sessions" "$(sql "SELECT coalesce(sum(downloaded), 0) FROM peers WHERE user_id = $id")" "$down"
    # upload_factor 200, download_factor 50, rounded down on every announce
    assert_eq "$role credited uploaded" "$(user_value "$role" uploaded)" $((up * 2))
    assert_between "$role credited downloaded" "$(user_value "$role" downloaded)" $((down / 2 - 10)) $((down / 2))
    assert_eq "$role activity" \
        "$(sql "SELECT real_uploaded || '/' || real_downloaded FROM torrent_activities WHERE torrent_id = 1 AND user_id = $id")" \
        "$up/$down"
    fail "$role was inconsistent for 10 seconds"
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
    # With SHORT_ACTIVE_PEER_TTL, peers get reaped 10 seconds after their last announce, so they
    # all have to keep announcing to be counted at the same time. Transmission 4.0 only accepts
    # a reannounce 2 minutes after the first announce of a session (a restarted leecher is a new one).
    wait_sql "torrent seeders/leechers/times_completed" "$torrent_counts_query" "$counts" \
        client_reannounce "$@"
    for role in "$@"; do
        assert_user_consistent "$role"
        wait_sql "$role active peer" \
            "SELECT host(p.ip), p.port, p.seeder, p.left, starts_with(p.agent, '$agent_prefix')
             FROM peers p JOIN users u ON u.id = p.user_id WHERE u.username = 'e2e_$role' AND p.active" \
            "$(container_ip "$role")|6881|t|0|t" client reannounce "$role"
    done
    assert_eq "no announce errors" "$(sql "SELECT count(*) FROM announce_errors")" 0
}

# usage: assert_seeder_leecher_bounds [max transferred]
assert_seeder_leecher_bounds() {
    local max=${1:-$MAX_TRANSFERRED}
    assert_between "seeder uploaded" "$(user_value seeder real_uploaded)" "$SIZE" "$max"
    assert_between "leecher downloaded" "$(user_value leecher real_downloaded)" "$SIZE" "$max"
    assert_eq "leecher nothing uploaded" "$(user_value leecher real_uploaded)" 0
    assert_eq "seeder nothing downloaded" "$(user_value seeder real_downloaded)" 0
}

assert_payload() {
    cmp -s "$WORK_DIR/seeder/downloads/payload.bin" "$WORK_DIR/$1/downloads/payload.bin" ||
        fail "payload downloaded by $1 differs"
}

### Scenarios, the same for every client
# They drive the clients through `client <action> <role> [argument...]`, which calls the
# `<CLIENT>_<action>` function of the client under test:
#   prepare <role>              writes the configuration of the client, before it starts
#   wait_ready <role>           waits until the client can be controlled
#   add <role>                  adds the torrent of the role, saved in /downloads
#   progress <role>             prints the progress of the torrent, between 0 and 1, -1 if missing
#   reannounce <role>           forces an announce
#   download_limit <role> <B/s> limits the download speed, 0 to remove the limit
#   upload_limit <role> <B/s>   limits the upload speed
#   stop <role>                 stops the torrent
#   tracker_message <role>      prints what the client shows about the tracker
client() { "${CLIENT}_$1" "${@:2}"; }

client_reannounce() {
    local role
    for role in "$@"; do
        client reannounce "$role"
    done
}

# Waits until the torrent progress of a client reaches the given fraction, running an optional
# command before each try
# usage: wait_progress <role> <fraction> [command...]
wait_progress() {
    local role=$1 min=$2
    shift 2
    for _ in $(seq 240); do
        [ $# -gt 0 ] && "$@"
        python3 -c "import sys; sys.exit(not $(client progress "$role") >= $min)" && return 0
        sleep 0.5
    done
    fail "$role did not reach progress $min, at $(client progress "$role")"
}

# Fresh stack with the given clients up and the seeder seeding
# usage: start <role...> [-- tracker configuration override...]
start() {
    local roles=() role
    while [ $# -gt 0 ] && [ "$1" != -- ]; do
        roles+=("$1")
        shift
    done
    [ $# -gt 0 ] && shift
    setup "$@"
    for role in "${roles[@]}"; do
        client prepare "$role"
    done
    compose up -d "${roles[@]}"
    for role in "${roles[@]}"; do
        client wait_ready "$role"
    done
    client add seeder
    wait_progress seeder 1
    # Otherwise the leecher may announce first, get no peer, and only retry after the interval
    wait_sql "seeder peer active" \
        "SELECT p.active FROM peers p JOIN users u ON u.id = p.user_id WHERE u.username = 'e2e_seeder' AND p.seeder" t \
        client reannounce seeder
}

# Starts the leecher slowly, and waits until it got part of the payload
leecher_partially_downloaded() {
    client download_limit leecher $((2 * 1024 * 1024))
    client add leecher
    wait_progress leecher 0.3
}

# With SHORT_ACTIVE_PEER_TTL, the seeder gets reaped while the leecher restarts or is frozen, and
# the tracker gives no inactive peer: the leecher would only find the seeder once one of them
# announces again on its own, maybe after announce_max
leecher_finish() {
    client download_limit leecher 0
    wait_progress leecher 1 client reannounce seeder
    assert_payload leecher
}

# usage: run_scenarios <name of the client and version> <user agent prefix>
run_scenarios() {
    local name=$1 agent_prefix=$2 role

    echo "=== $name: transfer then stop"
    start seeder leecher
    client add leecher
    wait_progress leecher 1
    assert_payload leecher
    assert_all_reported "$agent_prefix" 2/0/1 seeder leecher
    assert_seeder_leecher_bounds
    client stop leecher
    wait_sql "leecher peer inactive after stop" \
        "SELECT p.active FROM peers p JOIN users u ON u.id = p.user_id WHERE u.username = 'e2e_leecher'" "f"
    wait_sql "seeders after stop" "SELECT seeders || '/' || leechers FROM torrents WHERE id = 1" "1/0"

    echo "=== $name: leecher restarted mid-download"
    # Some clients (qBittorrent 4.4) send no stopped event when shutting down, their previous
    # session is only counted out once it gets reaped
    start seeder leecher -- "${SHORT_ACTIVE_PEER_TTL[@]}"
    leecher_partially_downloaded
    compose restart -t 30 leecher
    client wait_ready leecher
    leecher_finish
    assert_all_reported "$agent_prefix" 2/0/1 seeder leecher
    # some clients (qBittorrent 4.4) also lose what they downloaded, and get it again
    assert_seeder_leecher_bounds $((SIZE + MAX_TRANSFERRED))

    echo "=== $name: swarm of one seeder and two leechers"
    start seeder leecher leecher2
    # slow seeder, so that the leechers get pieces from each other
    client upload_limit seeder $((1024 * 1024))
    client add leecher
    client add leecher2
    wait_progress leecher 1
    wait_progress leecher2 1
    assert_payload leecher
    assert_payload leecher2
    assert_all_reported "$agent_prefix" 3/0/2 seeder leecher leecher2
    for role in leecher leecher2; do
        assert_between "$role downloaded" "$(user_value $role real_downloaded)" "$SIZE" "$MAX_TRANSFERRED"
    done
    [ $(($(user_value leecher real_uploaded) + $(user_value leecher2 real_uploaded))) -gt 0 ] ||
        fail "the leechers did not upload to each other"

    echo "=== $name: leecher killed mid-download"
    start seeder leecher -- "${SHORT_ACTIVE_PEER_TTL[@]}"
    leecher_partially_downloaded
    compose kill leecher
    # the seeder keeps announcing, the leecher gets marked as inactive
    wait_sql "killed leecher reaped" "$torrent_counts_query" 1/0/0 client reannounce seeder
    # A new container, like a reboot after the crash. Restarting the killed one reuses its pid, so
    # qBittorrent sees its own stale lock file as another running instance and exits.
    compose rm -f leecher
    compose up -d leecher
    client wait_ready leecher
    leecher_finish
    # What the killed session downloaded was never announced, and the client may have lost it
    # too, without resume data saved before the crash, so the seeder may have sent it twice
    assert_all_reported "$agent_prefix" 2/0/1 seeder leecher
    assert_between "seeder uploaded" "$(user_value seeder real_uploaded)" "$SIZE" $((SIZE + MAX_TRANSFERRED))

    echo "=== $name: leecher frozen mid-download, then back"
    start seeder leecher -- "${SHORT_ACTIVE_PEER_TTL[@]}"
    leecher_partially_downloaded
    # like a suspended laptop: no announce and no stopped event, then the same session resumes
    compose pause leecher
    wait_sql "frozen leecher marked inactive" "$torrent_counts_query" 1/0/0 client reannounce seeder
    # a few more expiry passes, still far from inactive_peer_ttl
    for _ in $(seq 5); do
        client reannounce seeder
        sleep 1
    done
    compose unpause leecher
    leecher_finish
    assert_all_reported "$agent_prefix" 2/0/1 seeder leecher
    # the requests of the frozen leecher time out, some clients (Transmission 4.0) request the same
    # blocks again and get them twice
    assert_seeder_leecher_bounds $((SIZE + MAX_TRANSFERRED))

    echo "=== $name: client not in the whitelist"
    setup 'ARCADIA_TRACKER__ALLOWED_TORRENT_CLIENTS=[XX]'
    client prepare leecher
    compose up -d leecher
    client wait_ready leecher
    client add leecher
    wait_sql "whitelist error recorded" \
        "SELECT error_code FROM announce_errors WHERE user_id = $(user_id leecher)" torrent_client_not_in_whitelist
    for _ in $(seq 30); do
        client tracker_message leecher | grep -q "torrent client not in whitelist" && break
        sleep 1
    done
    client tracker_message leecher | grep -q "torrent client not in whitelist" ||
        fail "client does not show the whitelist error: $(client tracker_message leecher)"
    assert_eq "no peer for rejected client" "$(sql "SELECT count(*) FROM peers")" 0
    assert_eq "torrent counts untouched" "$(sql "$torrent_counts_query")" 0/0/0

    echo "=== $name: ok"
}

### qBittorrent (official qbittorrent-nox image), controlled through its web API

# Host port of the web API of each client, published on a random port and looked up whenever the
# client (re)starts
declare -A QBT_PORTS=()

qbt() {
    local role=$1 endpoint=$2
    shift 2
    curl -sf "http://127.0.0.1:${QBT_PORTS[$role]}/api/v2/$endpoint" "$@"
}

qbt_prepare() {
    mkdir -p "$WORK_DIR/$1/config/qBittorrent/config"
    cp qBittorrent.conf "$WORK_DIR/$1/config/qBittorrent/config/"
}

qbt_wait_ready() {
    for _ in $(seq 60); do
        QBT_PORTS[$1]=$(compose port "$1" 8080 2>/dev/null | cut -d: -f2)
        [ -n "${QBT_PORTS[$1]}" ] && qbt "$1" app/version >/dev/null 2>&1 && return 0
        sleep 1
    done
    fail "$1 web API not reachable"
}

qbt_add() { qbt "$1" torrents/add -F "torrents=@$WORK_DIR/$1.torrent" -F savepath=/downloads >/dev/null; }
qbt_progress() { { qbt "$1" "torrents/info?hashes=$INFO_HASH" || echo '[]'; } | python3 -c 'import json,sys; t=json.load(sys.stdin); print(t[0]["progress"] if t else -1)'; }
qbt_reannounce() { qbt "$1" torrents/reannounce -d "hashes=$INFO_HASH"; }
qbt_download_limit() { qbt "$1" transfer/setDownloadLimit -d "limit=$2"; }
qbt_upload_limit() { qbt "$1" transfer/setUploadLimit -d "limit=$2"; }
qbt_stop() { qbt "$1" "$QBT_STOP_ENDPOINT" -d "hashes=$INFO_HASH"; }
qbt_tracker_message() { qbt "$1" "torrents/trackers?hash=$INFO_HASH"; }

test_qbittorrent() {
    local version=$1
    CLIENT=qbt
    CLIENT_IMAGE=qbittorrentofficial/qbittorrent-nox:$version
    # qBittorrent 4.x calls stopping a torrent pausing it
    QBT_STOP_ENDPOINT=torrents/stop
    [[ $version == 4.* ]] && QBT_STOP_ENDPOINT=torrents/pause
    run_scenarios "qBittorrent $version" "qBittorrent/"
}

### Transmission (linuxserver image), controlled with transmission-remote inside the container

trc() { compose exec -T "$1" transmission-remote "${@:2}"; }

# announce-ip, ignored by the tracker, makes Transmission send a single announce. Otherwise 4.0
# sends one over IPv4 and one over IPv6, and without IPv6 it treats every announce as failed:
# it retries every 20 seconds, ignores reannounce requests and hangs on shutdown.
tr_prepare() {
    cp settings.json "$WORK_DIR/$1/config/"
    # A leecher reaped, then back on the same address, would be refused by a Transmission 4.0
    # seeder:
    # - over µTP, the seeder still holds the previous session, the connection times out and the
    #   leecher gives up on the seeder
    # - encrypted, when the tracker reports no leecher, the seeder takes every peer it knows for
    #   a seed, and refuses the encrypted connections of seeds
    if [[ $CLIENT_IMAGE == *:4.0.* ]]; then
        sed -i '1a\    "utp-enabled": false,\n    "encryption": 0,' "$WORK_DIR/$1/config/settings.json"
    fi
}

tr_wait_ready() {
    for _ in $(seq 60); do
        trc "$1" -l >/dev/null 2>&1 && return 0
        sleep 1
    done
    fail "$1 RPC not reachable"
}

tr_add() {
    cp "$WORK_DIR/$1.torrent" "$WORK_DIR/$1/config/"
    trc "$1" -a "/config/$1.torrent" -w /downloads >/dev/null
}

tr_progress() {
    { trc "$1" -t "$INFO_HASH" -i 2>/dev/null || true; } |
        python3 -c 'import re,sys; m=re.search(r"Percent Done: ([0-9.]+)%", sys.stdin.read()); print(float(m.group(1)) / 100 if m else -1)'
}

tr_reannounce() { trc "$1" -t "$INFO_HASH" --reannounce >/dev/null; }
# in kB/s, 1 kB being 1000 bytes
tr_download_limit() { if [ "$2" -eq 0 ]; then trc "$1" -D >/dev/null; else trc "$1" -d $(($2 / 1000)) >/dev/null; fi; }
tr_upload_limit() { trc "$1" -u $(($2 / 1000)) >/dev/null; }
tr_stop() { trc "$1" -t "$INFO_HASH" --stop >/dev/null; }
tr_tracker_message() { trc "$1" -t "$INFO_HASH" -it; }

test_transmission() {
    local version=$1
    CLIENT=tr
    CLIENT_IMAGE=linuxserver/transmission:$version
    run_scenarios "Transmission $version" "Transmission/"
}

### Tests, one per supported client version

# Latest patch of each minor version with an official image, older ones have none
qbittorrent-4.4.5() { test_qbittorrent 4.4.5-1; }
qbittorrent-4.5.5() { test_qbittorrent 4.5.5-1; }
qbittorrent-4.6.7() { test_qbittorrent 4.6.7-1; }
qbittorrent-5.0.5() { test_qbittorrent 5.0.5-1; }
qbittorrent-5.1.4() { test_qbittorrent 5.1.4-2; }
qbittorrent-5.2.4() { test_qbittorrent 5.2.4-1; }
qbittorrent-5.3.0() { test_qbittorrent 5.3.0-2; }
# Every version with an image
transmission-4.0.6() { test_transmission 4.0.6; }
transmission-4.1.0() { test_transmission 4.1.0; }
transmission-4.1.1() { test_transmission 4.1.1; }
transmission-4.1.2() { test_transmission 4.1.2; }
transmission-4.1.3() { test_transmission 4.1.3; }

TESTS=(
    qbittorrent-4.4.5 qbittorrent-4.5.5 qbittorrent-4.6.7
    qbittorrent-5.0.5 qbittorrent-5.1.4 qbittorrent-5.2.4 qbittorrent-5.3.0
    transmission-4.0.6 transmission-4.1.0 transmission-4.1.1 transmission-4.1.2 transmission-4.1.3
)

# A single test, started by the loop below
if [ -n "${E2E_TEST:-}" ]; then
    trap teardown EXIT
    # so that the teardown also runs when the parent gets interrupted
    trap 'exit 1' INT TERM
    "$E2E_TEST"
    exit
fi

# Built once here, the projects of the tests only use the images
WORK_DIR=$(mktemp -d)
touch "$WORK_DIR/tracker.env"
CLIENT_IMAGE=unused compose build init_db tracker
rm -rf "$WORK_DIR"
WORK_DIR=""

LOG_DIR=${LOG_DIR:-$(mktemp -d)}
mkdir -p "$LOG_DIR"
echo "logs in $LOG_DIR"
trap 'kill $(jobs -p) 2>/dev/null; wait' INT TERM

declare -A PIDS=()
RUN=("${@:-${TESTS[@]}}")
for test in "${RUN[@]}"; do
    while [ "$(jobs -rp | wc -l)" -ge "${JOBS:-$(nproc)}" ]; do
        wait -n || true
    done
    E2E_TEST=$test COMPOSE_PROJECT_NAME=arcadia-tracker-e2e-${test//./-} "$SELF" >"$LOG_DIR/$test.log" 2>&1 &
    PIDS[$test]=$!
done

failed=()
for test in "${RUN[@]}"; do
    if wait "${PIDS[$test]}"; then
        echo "ok    $test"
    else
        echo "FAIL  $test"
        failed+=("$test")
    fi
done
for test in "${failed[@]}"; do
    echo "===== $test" >&2
    cat "$LOG_DIR/$test.log" >&2
done
[ ${#failed[@]} -eq 0 ]
