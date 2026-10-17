# Shared by backup.sh and restore.sh, sourced from the repository root. Reads the `backup` section
# of the configuration file, documented in config.example.yml.
set -euo pipefail
. scripts/config_value.sh

cfg() { config_value backup "$1"; }
MODE=$(cfg mode)
VOLUMES=$(cfg volumes)
PATHS=$(cfg paths)
REMOTES=$(cfg remote_repo)
RESTIC_IMAGE=restic/restic:0.19.1
# The gitignored configuration files of the repository, skipped when absent.
CONFIG_FILES="config.yml ergo/ergo-conf.yaml ergo/ergo.motd kiwiirc/config.json compose.override.yml"

case $MODE in docker | host) ;; *) echo "backup.mode must be docker or host, not '$MODE'" >&2; exit 1 ;; esac

# Where restic sees the repo, dumps and password. In docker mode they are fixed: compose bind-mounts
# the host BACKUP_DIR and password file (set in .env, the only knobs) at these points inside backup_cron,
# so config.yml does not repeat them. backup.sh runs in the container and uses the mount paths directly;
# restore.sh runs on the host and also needs the host side of the mounts (read from .env the way compose
# does) to bind-mount them for the restic image and to read the restored dumps back. In host mode restic
# runs on the host, so the paths come from config.yml and the host/config paths are the same.
BACKUP_MOUNT=/var/backups/arcadia
PASSWORD_MOUNT=/root/.arcadia-restic-password
if [ "$MODE" = docker ]; then
    REPO=$BACKUP_MOUNT/repo
    DUMP_DIR=$BACKUP_MOUNT/dumps
    PASSWORD_FILE=$PASSWORD_MOUNT
    DUMP_DIR_HOST=$DUMP_DIR
    PASSWORD_FILE_HOST=$PASSWORD_FILE
    if [ -z "${ARCADIA_BACKUP_CONTAINER:-}" ]; then
        if [ -f .env ]; then set -a +u; . ./.env; set +a -u; fi
        : "${BACKUP_DIR:=$BACKUP_MOUNT}"
        : "${RESTIC_PASSWORD_FILE:=$PASSWORD_MOUNT}"
        DUMP_DIR_HOST=$BACKUP_DIR/dumps
        PASSWORD_FILE_HOST=$RESTIC_PASSWORD_FILE
    fi
else
    REPO=$(cfg repo)
    DUMP_DIR=$(cfg dump_dir)
    PASSWORD_FILE=$(cfg password_file)
    DUMP_DIR_HOST=$DUMP_DIR
    PASSWORD_FILE_HOST=$PASSWORD_FILE
    for p in "$REPO" "$DUMP_DIR"; do
        case $p in /?*) ;; *) echo "backup.repo and backup.dump_dir must be absolute paths" >&2; exit 1 ;; esac
    done
fi
# restore.sh runs restic from its image on the host and must name the real, project-prefixed
# volumes. Default the project the way compose does: the lowercased repo-dir basename, invalid
# characters stripped. (Not needed in the backup container, which uses local restic.)
if [ "$MODE" = docker ] && [ -z "${ARCADIA_BACKUP_CONTAINER:-}" ]; then
    : "${COMPOSE_PROJECT_NAME:=$(basename "$PWD" | tr '[:upper:]' '[:lower:]' | tr -cd 'a-z0-9_-')}"
    export COMPOSE_PROJECT_NAME
fi
for _r in $REMOTES; do
    case $_r in /* | *:*) ;; *) echo "each backup.remote_repo entry must be absolute or a restic backend URL, not '$_r'" >&2; exit 1 ;; esac
done
[ -r "$PASSWORD_FILE_HOST" ] || { echo "cannot read the restic password file '$PASSWORD_FILE_HOST'" >&2; exit 1; }
# Without retention flags restic forget refuses to prune, which would abort every run after the
# snapshot and before the copy to remote_repo.
[ -n "$(cfg keep)" ] || { echo "backup.keep must hold restic forget flags, e.g. --keep-daily 7 --keep-weekly 4" >&2; exit 1; }
# Without a data target the snapshot holds the dumps and the configuration files only, which looks
# like a backup of an installation whose images and ircd are gone.
if [ "$MODE" = docker ]; then
    [ -n "$VOLUMES" ] || { echo "backup.volumes must list the volumes to back up in docker mode" >&2; exit 1; }
else
    [ -n "$PATHS" ] || { echo "backup.paths must list the data directories to back up in host mode" >&2; exit 1; }
fi

# What is backed up raw, as restic sees it: the volumes mounted at /data/<name> by compose or the host paths.
data_targets() {
    local t
    if [ "$MODE" = docker ]; then for t in $VOLUMES; do echo "/data/$t"; done
    else for t in $PATHS; do echo "$t"; done; fi
}

# restic on the local repository. Inside backup_cron (ARCADIA_BACKUP_CONTAINER, checked first because
# MODE is docker there too) it runs locally; restore.sh on a docker host runs it from its image; host
# mode runs it locally.
restic() {
    if [ -n "${ARCADIA_BACKUP_CONTAINER:-}" ]; then
        # inside backup_cron: volumes are compose-mounted at /data/<name>; repo/dump_dir same-path.
        RESTIC_REPOSITORY="$REPO" RESTIC_PASSWORD_FILE="$PASSWORD_FILE" \
            RESTIC_FROM_PASSWORD_FILE="$PASSWORD_FILE" \
            AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-}" AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-}" \
            command restic --retry-lock 30m "$@"
    elif [ "$MODE" = docker ]; then
        # restore.sh on the host: restic from its image, mounting the real (project-prefixed) volumes
        # and bind-mounting the host BACKUP_DIR and password file at the same fixed points compose uses,
        # so the restic paths (repo, dump_dir) match an in-container backup. repo and dump_dir live
        # under BACKUP_MOUNT, so the single BACKUP_DIR mount covers both.
        # A tty when there is one, otherwise restic prints no progress during long restores.
        local mounts=() v tty=()
        for v in $VOLUMES; do mounts+=(-v "${COMPOSE_PROJECT_NAME}_$v:/data/$v"); done
        if [ -t 1 ]; then tty=(-t); fi
        docker run --rm "${tty[@]}" -e RESTIC_REPOSITORY="$REPO" -e RESTIC_PASSWORD_FILE="$PASSWORD_FILE" \
            -e RESTIC_FROM_PASSWORD_FILE="$PASSWORD_FILE" -e AWS_ACCESS_KEY_ID -e AWS_SECRET_ACCESS_KEY \
            -v "$PASSWORD_FILE_HOST:$PASSWORD_FILE:ro" -v "$BACKUP_DIR:$BACKUP_MOUNT" "${mounts[@]}" \
            "$RESTIC_IMAGE" --retry-lock 30m "$@"
    else
        RESTIC_REPOSITORY="$REPO" RESTIC_PASSWORD_FILE="$PASSWORD_FILE" \
            RESTIC_FROM_PASSWORD_FILE="$PASSWORD_FILE" command restic --retry-lock 30m "$@"
    fi
}

# Only what the scripts write: dump_dir may hold other files. DUMP_DIR_HOST so restore.sh cleans the
# dumps where they land on the host (equal to DUMP_DIR in the container and in host mode).
clean_dumps() { rm -rf "$DUMP_DIR_HOST"/{arcadia,ergo_database,chevereto_database}.sql "$DUMP_DIR_HOST"/.tmp.* "$DUMP_DIR_HOST/config"; }
