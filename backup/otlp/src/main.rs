//! Build and POST OTLP/HTTP metrics for an arcadia backup run.
//!
//! Best effort: any failure is swallowed so telemetry never breaks the backup. Invoked by backup.sh
//! with restic's `backup --json` summary line on stdin (empty if the run failed before the snapshot):
//!
//!     printf '%s' "$RESTIC_SUMMARY_JSON" | arcadia-otlp <exit_code> <duration_ms>
//!
//! Self-test, no network:  arcadia-otlp --self-test
//!
//! The collector base URL is ARCADIA_BACKUP_OTLP_HTTP (backup.sh sets it from telemetry.otlp_endpoint,
//! host with the HTTP port 4318; default http://otel-lgtm:4318, the bundled otel-lgtm, for standalone
//! runs). otel-lgtm listens on 4317 gRPC and 4318 HTTP. Metrics are plain gauges -- last-run values,
//! which is what a nightly job needs. restic summary fields: https://restic.readthedocs.io.
use serde_json::{json, Value};
use std::io::Read;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SERVICE: &str = "arcadia-backup";

/// restic `backup --json` summary field -> (otel metric name, unit, integer?).
const RESTIC_METRICS: &[(&str, &str, &str, bool)] = &[
    ("files_new", "arcadia.backup.files_new", "1", true),
    ("files_changed", "arcadia.backup.files_changed", "1", true),
    ("files_unmodified", "arcadia.backup.files_unmodified", "1", true),
    ("data_added", "arcadia.backup.data_added_bytes", "By", true),
    ("data_added_packed", "arcadia.backup.data_added_packed_bytes", "By", true),
    ("total_files_processed", "arcadia.backup.total_files_processed", "1", true),
    ("total_bytes_processed", "arcadia.backup.total_bytes_processed", "By", true),
    ("total_duration", "arcadia.backup.restic_duration_seconds", "s", false),
];

/// One OTLP gauge metric with a single data point. `number` is pre-keyed as OTLP wants it: an
/// integer gauge carries its value as a string under "asInt", a double as a JSON number under
/// "asDouble".
fn gauge(name: &str, unit: &str, now: u128, attrs: &Value, number_key: &str, number: Value) -> Value {
    json!({
        "name": name,
        "unit": unit,
        "gauge": {"dataPoints": [{
            "timeUnixNano": now.to_string(),
            "attributes": attrs,
            number_key: number,
        }]},
    })
}

fn build_payload(exit_code: i64, duration_ms: i64, summary: &Value) -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let status = if exit_code == 0 { "ok" } else { "err" };
    let attrs = json!([
        {"key": "status", "value": {"stringValue": status}},
        {"key": "host", "value": {"stringValue": "arcadia"}},
    ]);
    let mut metrics = vec![
        gauge("arcadia.backup.success", "1", now, &attrs, "asInt",
              json!(i64::from(exit_code == 0).to_string())),
        gauge("arcadia.backup.duration_ms", "ms", now, &attrs, "asInt",
              json!(duration_ms.to_string())),
    ];
    // restic summary metrics only exist on a successful snapshot; a failed run reports status alone.
    for &(field, name, unit, is_int) in RESTIC_METRICS {
        let Some(v) = summary.get(field) else { continue };
        if is_int {
            if let Some(i) = v.as_i64() {
                metrics.push(gauge(name, unit, now, &attrs, "asInt", json!(i.to_string())));
            }
        } else if let Some(f) = v.as_f64() {
            metrics.push(gauge(name, unit, now, &attrs, "asDouble", json!(f)));
        }
    }
    json!({"resourceMetrics": [{
        "resource": {"attributes": [{"key": "service.name", "value": {"stringValue": SERVICE}}]},
        "scopeMetrics": [{"metrics": metrics}],
    }]})
}

/// The restic summary object parsed from `text`, or `{}` when there is none (e.g. the backup failed).
/// backup.sh already greps out the single `message_type":"summary"` line, so `text` is that line (or
/// empty); just parse it.
fn parse_summary(text: &str) -> Value {
    let text = text.trim();
    if text.is_empty() {
        return json!({});
    }
    serde_json::from_str(text).unwrap_or_else(|_| json!({}))
}

fn read_summary() -> Value {
    let mut text = String::new();
    if std::io::stdin().read_to_string(&mut text).is_err() {
        return json!({});
    }
    parse_summary(&text)
}

fn post(payload: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let base = std::env::var("ARCADIA_BACKUP_OTLP_HTTP")
        .unwrap_or_else(|_| "http://otel-lgtm:4318".to_string());
    let url = format!("{}/v1/metrics", base.trim_end_matches('/'));
    ureq::post(&url)
        .config()
        .timeout_global(Some(Duration::from_secs(5)))
        .build()
        .header("Content-Type", "application/json")
        .send(&serde_json::to_vec(payload)?)?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() == 1 && args[0] == "--self-test" {
        return self_test();
    }
    // Best effort throughout: bad args default to 0, and a failed POST is swallowed, so telemetry
    // never changes the backup's exit status.
    let exit_code = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let duration_ms = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let _ = post(&build_payload(exit_code, duration_ms, &read_summary()));
}

fn self_test() {
    let summary = json!({
        "message_type": "summary", "files_new": 3, "files_changed": 1, "files_unmodified": 100,
        "data_added": 4096, "data_added_packed": 2048, "total_files_processed": 104,
        "total_bytes_processed": 1048576, "total_duration": 1.5,
    });
    let ok = build_payload(0, 1234, &summary);
    let ok_metrics = ok["resourceMetrics"][0]["scopeMetrics"][0]["metrics"].as_array().unwrap();
    let find = |m: &[Value], n: &str| {
        m.iter().find(|x| x["name"] == n).unwrap()["gauge"]["dataPoints"][0].clone()
    };
    assert_eq!(find(ok_metrics, "arcadia.backup.success")["asInt"], "1");
    assert_eq!(find(ok_metrics, "arcadia.backup.data_added_bytes")["asInt"], "4096");
    assert_eq!(find(ok_metrics, "arcadia.backup.restic_duration_seconds")["asDouble"], json!(1.5));

    // a failed run: no restic summary, success=0, numeric restic metrics absent.
    let fail = build_payload(2, 50, &json!({}));
    let fail_metrics = fail["resourceMetrics"][0]["scopeMetrics"][0]["metrics"].as_array().unwrap();
    let names: std::collections::BTreeSet<&str> =
        fail_metrics.iter().map(|m| m["name"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        ["arcadia.backup.duration_ms", "arcadia.backup.success"].into_iter().collect()
    );
    assert_eq!(find(fail_metrics, "arcadia.backup.success")["asInt"], "0");

    assert_eq!(parse_summary(""), json!({}));
    assert_eq!(parse_summary("not json"), json!({}));
    println!("otlp self-test passed");
}
