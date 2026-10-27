#!/usr/bin/env bash
# End to end tests of the tracker with real torrent clients, one test per supported client version.
# usage: ./run.sh [test name...]   (all tests when none is given)
set -euo pipefail
cd "$(dirname "$0")"

FIXTURES=../arcadia_tracker/tests/fixtures
SEEDER_PASSKEY=5eede75eede75eede75eede75eede75e
LEECHER_PASSKEY=1eec4e121eec4e121eec4e121eec4e12
SIZE=$((16 * 1024 * 1024))
# Pieces in flight are discarded when a client restarts and get transferred again. More than
# this means some bytes were counted twice.
MAX_TRANSFERRED=$((SIZE + SIZE / 4))

export HOST_UID HOST_GID WORK_DIR CLIENT_IMAGE
HOST_UID=$(id -u)
HOST_GID=$(id -g)

compose() { docker compose "$@"; }
sql() { compose exec -T db psql -U arcadia -d arcadia -tA -c "$1"; }
fail() {
    echo "FAIL: $*" >&2
    exit 1
}

# Polls a query until it returns the expected value
wait_sql() {
    local description=$1 query=$2 expected=$3 actual=""
    for _ in $(seq 60); do
        actual=$(sql "$query")
        [ "$actual" = "$expected" ] && return 0
        sleep 1
    done
    fail "$description: expected '$expected', got '$actual'"
}

assert_eq() { [ "$2" = "$3" ] || fail "$1: expected '$3', got '$2'"; }
assert_between() { [ "$2" -ge "$3" ] && [ "$2" -le "$4" ] || fail "$1: expected $3..$4, got $2"; }

user_id() { sql "SELECT id FROM users WHERE username = '$1'"; }
container_ip() { docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$(compose ps -q "$1")"; }

# Starts the db, the tracker and both clients, with a fresh database and payload
setup() {
    compose down -v --remove-orphans >/dev/null 2>&1 || true
    [ -n "${WORK_DIR:-}" ] && rm -rf "$WORK_DIR"
    WORK_DIR=$(mktemp -d)
    mkdir -p "$WORK_DIR"/{seeder,leecher}/{config,downloads}
    head -c "$SIZE" /dev/urandom >"$WORK_DIR/seeder/downloads/payload.bin"
    INFO_HASH=$(python3 -I make_torrent.py "$WORK_DIR/seeder/downloads/payload.bin" \
        "http://tracker:8081/announce/$SEEDER_PASSKEY" "$WORK_DIR/seeder.torrent")
    python3 -I make_torrent.py "$WORK_DIR/seeder/downloads/payload.bin" \
        "http://tracker:8081/announce/$LEECHER_PASSKEY" "$WORK_DIR/leecher.torrent" >/dev/null

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
        compose logs tracker seeder leecher >&2
    fi
    compose down -v --remove-orphans >/dev/null 2>&1 || true
    [ -n "${WORK_DIR:-}" ] && rm -rf "$WORK_DIR"
}
trap teardown EXIT

# Same assertions for every client, once the leecher completed.
# usage: assert_transfer_reported <agent prefix> <command making both clients reannounce>
assert_transfer_reported() {
    local agent_prefix=$1 reannounce=$2 seeder leecher
    seeder=$(user_id e2e_seeder)
    leecher=$(user_id e2e_leecher)

    # the seeder only reports its upload on its next announce, and clients may skip a forced
    # reannounce that comes too early
    for _ in $(seq 30); do
        $reannounce
        sleep 2
        [ "$(sql "SELECT real_uploaded >= $SIZE FROM users WHERE id = $seeder")" = t ] && break
    done
    wait_sql "torrent seeders/leechers/times_completed" \
        "SELECT seeders || '/' || leechers || '/' || times_completed FROM torrents WHERE id = 1" "2/0/1"

    local up down
    up=$(sql "SELECT real_uploaded FROM users WHERE id = $seeder")
    down=$(sql "SELECT real_downloaded FROM users WHERE id = $leecher")
    echo "    reported: seeder uploaded $up, leecher downloaded $down, size $SIZE"
    assert_between "seeder real_uploaded" "$up" "$SIZE" "$MAX_TRANSFERRED"
    assert_between "leecher real_downloaded" "$down" "$SIZE" "$MAX_TRANSFERRED"
    # Every peer session (a new peer_id per client restart) is counted once, from the last
    # amounts the client announced in it. Both clients do not have to agree: data in flight when
    # the leecher stops is counted as uploaded by the seeder only.
    assert_eq "seeder uploaded = its sessions" "$(sql "SELECT sum(uploaded) FROM peers WHERE user_id = $seeder")" "$up"
    assert_eq "leecher downloaded = its sessions" "$(sql "SELECT sum(downloaded) FROM peers WHERE user_id = $leecher")" "$down"
    # upload_factor 200, download_factor 50
    assert_eq "seeder credited uploaded" "$(sql "SELECT uploaded FROM users WHERE id = $seeder")" $((up * 2))
    assert_between "leecher credited downloaded" "$(sql "SELECT downloaded FROM users WHERE id = $leecher")" \
        $((down / 2 - 2)) $((down / 2))
    assert_eq "leecher nothing uploaded" "$(sql "SELECT real_uploaded FROM users WHERE id = $leecher")" 0
    assert_eq "seeder nothing downloaded" "$(sql "SELECT real_downloaded FROM users WHERE id = $seeder")" 0

    assert_eq "leecher activity" \
        "$(sql "SELECT completed_at IS NOT NULL, real_downloaded FROM torrent_activities WHERE torrent_id = 1 AND user_id = $leecher")" \
        "t|$down"
    assert_eq "seeder activity" \
        "$(sql "SELECT real_uploaded FROM torrent_activities WHERE torrent_id = 1 AND user_id = $seeder")" "$up"

    local role ip
    for role in seeder leecher; do
        ip=$(container_ip $role)
        assert_eq "$role peer row" \
            "$(sql "SELECT host(p.ip), p.port, p.active, p.seeder, p.left, starts_with(p.agent, '$agent_prefix')
                    FROM peers p JOIN users u ON u.id = p.user_id WHERE u.username = 'e2e_$role' AND p.active")" \
            "$ip|6881|t|t|0|t"
    done

    assert_eq "no announce errors" "$(sql "SELECT count(*) FROM announce_errors")" 0
}

### qBittorrent (official qbittorrent-nox image)

qbt() {
    local role=$1 endpoint=$2
    shift 2
    local port=18080
    [ "$role" = leecher ] && port=18081
    curl -sf "http://127.0.0.1:$port/api/v2/$endpoint" "$@"
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

# Fresh stack with both clients up and the seeder seeding
qbt_start() {
    setup
    for role in seeder leecher; do
        mkdir -p "$WORK_DIR/$role/config/qBittorrent/config"
        cp qBittorrent.conf "$WORK_DIR/$role/config/qBittorrent/config/"
    done
    compose up -d seeder leecher
    qbt_wait_ready seeder
    qbt_wait_ready leecher
    qbt seeder torrents/add -F "torrents=@$WORK_DIR/seeder.torrent" -F savepath=/downloads >/dev/null
    qbt_wait_progress seeder 1
}

qbt_leecher_add() { qbt leecher torrents/add -F "torrents=@$WORK_DIR/leecher.torrent" -F savepath=/downloads >/dev/null; }

qbt_finish_and_assert() {
    qbt_wait_progress leecher 1
    cmp -s "$WORK_DIR/seeder/downloads/payload.bin" "$WORK_DIR/leecher/downloads/payload.bin" ||
        fail "downloaded payload differs"
    assert_transfer_reported "qBittorrent/" qbt_reannounce
}

qbt_reannounce() {
    qbt seeder torrents/reannounce -d "hashes=$INFO_HASH"
    qbt leecher torrents/reannounce -d "hashes=$INFO_HASH"
}

test_qbittorrent() {
    local version=$1 stop_endpoint=torrents/stop
    # qBittorrent 4.x calls stopping a torrent pausing it
    [[ $version == 4.* ]] && stop_endpoint=torrents/pause
    CLIENT_IMAGE=qbittorrentofficial/qbittorrent-nox:$version

    echo "=== qBittorrent $version: transfer then stop"
    qbt_start
    qbt_leecher_add
    qbt_finish_and_assert
    qbt leecher "$stop_endpoint" -d "hashes=$INFO_HASH"
    wait_sql "leecher peer inactive after stop" \
        "SELECT p.active FROM peers p JOIN users u ON u.id = p.user_id WHERE u.username = 'e2e_leecher'" "f"
    wait_sql "seeders after stop" "SELECT seeders || '/' || leechers FROM torrents WHERE id = 1" "1/0"

    echo "=== qBittorrent $version: leecher restarted mid-download"
    qbt_start
    # slow enough to restart the leecher halfway through
    qbt leecher transfer/setDownloadLimit -d limit=$((2 * 1024 * 1024))
    qbt_leecher_add
    qbt_wait_progress leecher 0.3
    compose restart -t 30 leecher
    qbt_wait_ready leecher
    qbt leecher transfer/setDownloadLimit -d limit=0
    qbt_finish_and_assert
    echo "=== qBittorrent $version: ok"
}

### Tests, one per supported client version

qbittorrent-4.6.7() { test_qbittorrent 4.6.7-1; }
qbittorrent-5.3.0() { test_qbittorrent 5.3.0-2; }

TESTS=(qbittorrent-4.6.7 qbittorrent-5.3.0)
for test in "${@:-${TESTS[@]}}"; do
    "$test"
done
