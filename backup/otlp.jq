# OTLP/HTTP metrics payload for a backup run, used by report_otlp in backup.sh:
#   printf '%s' "$RESTIC_SUMMARY_JSON" | jq -cRn --arg code 0 --arg ms 1234 --arg now <unix ns> -f backup/otlp.jq
# stdin is restic's `backup --json` summary line, empty when the run failed before the snapshot (the
# run then reports success and duration only). Metrics are plain gauges -- last-run values, which is
# what a nightly job needs. restic summary fields: https://restic.readthedocs.io.
(try (input | fromjson) catch {}) as $s
| [{key: "status", value: {stringValue: (if $code == "0" then "ok" else "err" end)}},
   {key: "host", value: {stringValue: "arcadia"}}] as $a
# OTLP carries an integer gauge as a string under "asInt", a double as a number under "asDouble".
| def g($n; $u; $k; $v): {name: $n, unit: $u, gauge: {dataPoints: [{timeUnixNano: $now, attributes: $a, ($k): $v}]}};
{resourceMetrics: [{
    resource: {attributes: [{key: "service.name", value: {stringValue: "arcadia-backup"}}]},
    scopeMetrics: [{metrics: (
        [g("arcadia.backup.success"; "1"; "asInt"; (if $code == "0" then "1" else "0" end)),
         g("arcadia.backup.duration_ms"; "ms"; "asInt"; $ms)]
        # restic summary field, metric name suffix, unit
        + ([["files_new", "files_new", "1"],
            ["files_changed", "files_changed", "1"],
            ["files_unmodified", "files_unmodified", "1"],
            ["data_added", "data_added_bytes", "By"],
            ["data_added_packed", "data_added_packed_bytes", "By"],
            ["total_files_processed", "total_files_processed", "1"],
            ["total_bytes_processed", "total_bytes_processed", "By"]]
           | map(select($s[.[0]] | type == "number")
                 | g("arcadia.backup." + .[1]; .[2]; "asInt"; $s[.[0]] | tostring)))
        + if $s.total_duration | type == "number"
          then [g("arcadia.backup.restic_duration_seconds"; "s"; "asDouble"; $s.total_duration)]
          else [] end
    )}]
}]}
