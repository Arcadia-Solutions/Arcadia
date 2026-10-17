# Backup

Backups are made with [restic](https://restic.net): every run is an encrypted, deduplicated snapshot,
so only what changed is stored and transferred.

- `backup/backup.sh` runs on the arcadia server. It dumps the databases (arcadia, ergo, chevereto), then
  snapshots the dumps, the configuration files (`config.yml`, `ergo/ergo-conf.yaml`, `ergo/ergo.motd`,
  `kiwiirc/config.json`, `compose.override.yml`) and the volumes (docker) or data directories (bare
  metal) into a local repository, and prunes it.
- `backup/cron.sh` runs `backup.sh` on the schedule of `backup.cron`, inside the `backup_cron` compose
  service (docker mode only).
- `backup/pull.sh` runs on a separate backup host. It copies the snapshots it does not have yet over
  ssh, so the backup host keeps its own history, which the arcadia server cannot reach or delete.
- `backup/restore.sh` restores a snapshot, on the same server or on a new one.

The database volumes are never copied raw (files of a running database are not consistent): the
dumps cover them.

## Requirements

- docker mode: nothing but docker. The backup image carries restic, `pg_dump` and `mariadb-dump`
  (no docker CLI, no docker socket). Restoring runs on the host and uses restic from its pinned image.
- host mode: [restic](https://restic.readthedocs.io/en/stable/020_installation.html) 0.17 or newer,
  `pg_dump`/`psql`, `mariadb-dump`/`mariadb` matching the server versions, and `curl` (backup/restore.sh
  uses it to check that the backend is stopped). The scripts run as root.
- backup host: restic 0.17 or newer, a checkout of this repository (or `backup/pull.sh` and
  `scripts/config_value.sh`).

## Configuration

Everything is in the `backup` section of `config.yml`, documented in `config.example.yml`. The
database credentials are read from the `database` section, and in docker mode the ergo and chevereto
credentials come from the environment of the `backup_cron` service.

Environment variables named `ARCADIA_<SECTION>__<KEY>` (upper-cased, `-` replaced by `_`, e.g.
`ARCADIA_DATABASE__PASSWORD`) override the values of `config.yml`. The `ARCADIA_` prefix and `__`
separator keep a `compose.override.yml` able to set credentials without touching `config.yml`.

In docker mode the backup location is set only by two `.env` settings (see `example.env`); the repo,
dumps and password file live inside them and `config.yml` does not repeat their paths (`backup.repo`,
`backup.dump_dir` and `backup.password_file` are host mode only, ignored in docker mode):

- `BACKUP_DIR` (default `/var/backups/arcadia`): host directory holding the repo and dumps,
  bind-mounted at the fixed path `/var/backups/arcadia` in the container. `restore.sh` runs on the host
  and reads `BACKUP_DIR` from `.env` to find the repository and dumps. Point it at the parent of the
  restic repository, which must be named `repo` (`$BACKUP_DIR/repo`).
- `RESTIC_PASSWORD_FILE`: host path of the password file, kept outside of `BACKUP_DIR`. It is
  bind-mounted at the fixed path `/root/.arcadia-restic-password`.

Create the repository password, and keep a copy somewhere safe, it is not part of the backups:

```bash
head -c 32 /dev/urandom | base64 > /root/.arcadia-restic-password
chmod 600 /root/.arcadia-restic-password
```

The repository is created by the first run of `backup/backup.sh`.

`keep` decides how many snapshots the server keeps. It must outlast the longest outage of the backup
host, or snapshots are pruned before being pulled. `remote_repo` optionally copies every snapshot to
other repositories: it is a space-separated list of push targets (local paths, `s3:`, `rest:`, `sftp:`,
any [restic backend](https://restic.readthedocs.io/en/stable/030_preparing_a_new_repo.html)).

## Scheduling

`backup.mode` decides how: in docker mode the `backup_cron` service runs the job on the schedule of
`backup.cron`, in host mode (or if you prefer the host's own tools) a systemd timer runs it.

### Docker: the backup_cron service

```bash
docker compose --profile backup up -d backup_cron
```

`backup.cron` holds the schedule in cron syntax, read in `backup.cron_timezone` (containers have no
timezone of their own). The job is `backup/backup.sh` itself, so everything the sections above
describe applies. The container joins the compose network and dumps `db` (postgres) and
`ergo_database`/`chevereto_database` (mariadb) over TCP, the mariadb ones as their per-database user,
not root. restic runs inside the container. There is no docker socket and no docker CLI: the image is
based on `postgres:18-alpine` (so `pg_dump` matches the `db` service) plus restic and the mariadb
client.

The service mounts the installation (read only), `BACKUP_DIR`, the password file and each volume of
`backup.volumes` at `/data/<name>` (read only). `config.yml` is the configuration, together with the
`.env` settings above.

To run a backup by hand:

```bash
docker compose --profile backup run --rm --entrypoint /arcadia/backup/backup.sh backup_cron
```

In docker mode `backup/backup.sh` cannot be run directly on the host: the database service names
resolve only on the compose network.

The output of a run, and the fact that it failed, are in the container log:

```bash
docker compose logs backup_cron
```

### systemd, on the host

`/etc/systemd/system/arcadia-backup.service`:

```ini
[Unit]
Description=Arcadia backup

[Service]
Type=oneshot
ExecStart=/opt/arcadia/backup/backup.sh
```

`/etc/systemd/system/arcadia-backup.timer`:

```ini
[Timer]
OnCalendar=*-*-* 03:00
Persistent=true

[Install]
WantedBy=timers.target
```

```bash
systemctl enable --now arcadia-backup.timer
```

A failing run exits with a non-zero code: use `OnFailure=` (or cron's mail) to be notified.

## Backup host

On the arcadia server, give the backup host's ssh key access to a `backup` user that can read the
repository and write its locks (restic creates new files group readable when the repository is):

```bash
useradd -m backup
chgrp -R backup /var/backups/arcadia/repo
chmod -R g+rX /var/backups/arcadia/repo
chmod g+w /var/backups/arcadia/repo/locks
find /var/backups/arcadia/repo -type d -exec chmod g+s {} +
```

On the backup host, write a configuration file, e.g. `/etc/arcadia-backup-pull.yml`:

```yaml
backup_pull:
  ssh: backup@arcadia.example.com
  source_repo: /var/backups/arcadia/repo
  # command run on the arcadia server before pulling, empty when it runs its own timer, e.g.
  # sudo /opt/arcadia/backup/backup.sh (needs a sudo rule for the backup user)
  trigger: ""
  repo: /srv/backups/arcadia
  # a copy of the arcadia repository password
  password_file: /root/.arcadia-restic-password
  # retention of the backup host, longer than the server's and keeping at least what it keeps
  keep: --keep-daily 30 --keep-weekly 12 --keep-monthly 12
```

and schedule `ARCADIA_CONFIG=/etc/arcadia-backup-pull.yml /opt/arcadia/backup/pull.sh` after the
server's backup, the same way as above (`Environment=ARCADIA_CONFIG=…` in the service). It fails when
no new snapshot was found, which also catches a server that stopped backing up. Missed days are
caught up: every missing snapshot is copied. The backup host's `keep` must keep at least every
snapshot the server still keeps, otherwise pruned snapshots are copied again and counted as new.

## Restore

```bash
backup/restore.sh            # latest snapshot
backup/restore.sh 1a2b3c4d   # a given snapshot, ids from `restic snapshots`
backup/restore.sh latest -y  # no confirmation
```

Stop the backup timer (and the backup host's trigger) before restoring, otherwise a backup can run in
the middle and `latest` changes.

Restore is run on the host, in both modes (in docker mode it uses the host's docker and restores the
volumes through the pinned restic image). It restores the configuration files, the volumes (or data directories) and the databases. In docker
mode it stops the running services first and starts them again at the end, and offers to recreate
the database volumes (`db_data`, `ergo_database`, `chevereto_database`): postgres and mariadb only
apply the passwords of `.env` when their volume is created, so a volume created before `.env` was
edited refuses them. Answer yes on a new server; `-y` leaves the volumes alone. In host mode, stop
arcadia, ergo and redis yourself first, and create the postgres role of the `database` section on a
new server. Run `backup/restore.sh` from the deployment checkout (the same directory the stack was started from), because in docker mode the restore derives the compose project name from the directory to locate the real data volumes.

### On a new server

1. Clone the repository, `cp config.example.yml config.yml`: only the `backup` section matters, it is
   overwritten by the restored `config.yml`. `dump_dir` must be the one used for the backups.
2. Put the password file in place.
3. From the backup host, copy the repository to the new server:
   ```bash
   restic -r sftp:root@new-server:/var/backups/arcadia/repo init --from-repo /srv/backups/arcadia --copy-chunker-params
   restic -r sftp:root@new-server:/var/backups/arcadia/repo copy --from-repo /srv/backups/arcadia
   ```
4. `backup/restore.sh`, then start the services.

### From docker to bare metal (or back)

Not scripted. Restore a volume into a directory with
`restic restore latest:/data/ergo_data --target /var/lib/ergo`, and load the dumps of
`dump_dir` with `psql` and `mariadb`.

## Telemetry

When `telemetry.otlp_endpoint` is set (the same switch the backend and tracker use, see
[Monitoring](./telemetry.md)), each run reports best-effort metrics over OTLP/HTTP: whether it
succeeded, its wall-clock duration, and restic's snapshot stats (files and bytes added, processed
totals, restic's own duration). The metrics are last-run gauges, labelled `status=ok|err`. Telemetry
never changes the backup's outcome: a failed export is swallowed.

The configured endpoint is a gRPC address (`host:4317`); the reporter reuses the host with the
OTLP/HTTP port `4318`.

The payload is built by `backup/otlp.jq` and posted with `curl`. Both are in the backup image. In host
mode, telemetry is skipped if `jq` or `curl` is missing, and the run is unaffected.

## Limits

- Paths and volume names cannot contain spaces, the crontab of the `backup_cron` service included: the
  job it schedules is one unquoted line.
- Restoring in docker mode creates the volumes outside of docker compose, which warns that they were
  "not created by Docker Compose"; it is harmless.
- `backup.volumes` takes unprefixed volume names, and each one must also be bind-mounted at
  `/data/<name>` in the `backup_cron` service of `compose.yml`: it is a two-place edit.
- A volume listed in `backup.volumes` but never populated is backed up empty: nothing checks that it
  exists.
- The mariadb dump is skipped only when the service is unreachable. A reachable service whose dump fails
  (bad credentials, for instance) fails the run.
