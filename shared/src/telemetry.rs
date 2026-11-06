use opentelemetry::metrics::Histogram;
use opentelemetry::trace::TracerProvider;
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use std::time::Instant;
use tracing::Instrument;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

static TRACER_PROVIDER: std::sync::OnceLock<SdkTracerProvider> = std::sync::OnceLock::new();
static LOGGER_PROVIDER: std::sync::OnceLock<SdkLoggerProvider> = std::sync::OnceLock::new();
static METER_PROVIDER: std::sync::OnceLock<SdkMeterProvider> = std::sync::OnceLock::new();

/// Initialize telemetry: stdout logging + optional OTLP export.
///
/// OTLP export is enabled only when `telemetry.otlp_endpoint` is set in the configuration file.
/// Without it, only the `fmt` (stdout) layer is active.
///
/// The log level is the `log_level` key of the calling service, and is overridden by the
/// `RUST_LOG` environment variable when it is set.
pub fn init_telemetry(
    config: &crate::config::TelemetryConfig,
    log_level: &str,
    service_name: &'static str,
) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));
    let fmt_layer = tracing_subscriber::fmt::layer();

    if let Some(endpoint) = &config.otlp_endpoint {
        let resource = Resource::builder().with_service_name(service_name).build();

        let otlp_trace_exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.as_str())
            .build()
            .expect("failed to create OTLP trace exporter");

        let tracer_provider = SdkTracerProvider::builder()
            .with_resource(resource.clone())
            .with_batch_exporter(otlp_trace_exporter)
            .build();

        let tracer = tracer_provider.tracer("arcadia");
        TRACER_PROVIDER
            .set(tracer_provider)
            .expect("tracer provider already set");

        // Metrics exporter
        let otlp_metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.as_str())
            .build()
            .expect("failed to create OTLP metrics exporter");

        let meter_provider = SdkMeterProvider::builder()
            .with_resource(resource.clone())
            .with_periodic_exporter(otlp_metrics_exporter)
            .build();

        METER_PROVIDER
            .set(meter_provider.clone())
            .expect("meter provider already set");

        opentelemetry::global::set_meter_provider(meter_provider.clone());

        let otel_trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        let otel_metrics_layer = tracing_opentelemetry::MetricsLayer::new(meter_provider);

        let otlp_log_exporter = opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.as_str())
            .build()
            .expect("failed to create OTLP log exporter");

        let logger_provider = SdkLoggerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(otlp_log_exporter)
            .build();

        let otel_log_layer = opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
            &logger_provider,
        );

        LOGGER_PROVIDER
            .set(logger_provider)
            .expect("logger provider already set");

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(otel_trace_layer)
            .with(otel_metrics_layer)
            .with(otel_log_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }
}

/// Histograms used to instrument periodic background tasks.
///
/// One pair (duration + rows processed) is registered per service via
/// [`PeriodicTaskInstruments::register`], and reused by all calls to
/// [`instrument_periodic_task`].
#[derive(Debug)]
pub struct PeriodicTaskInstruments {
    pub duration_ms: Histogram<f64>,
    pub rows_affected: Histogram<u64>,
}

impl PeriodicTaskInstruments {
    pub fn register(scope_name: &str, metric_prefix: &str) -> Self {
        let scope = opentelemetry::InstrumentationScope::builder(scope_name.to_string()).build();
        let meter = global::meter_with_scope(scope);
        Self {
            duration_ms: meter
                .f64_histogram(format!("{metric_prefix}.duration_ms"))
                .with_description("Duration of periodic task in milliseconds")
                .with_unit("ms")
                .build(),
            rows_affected: meter
                .u64_histogram(format!("{metric_prefix}.rows_affected"))
                .with_description("Number of rows affected by periodic task")
                .build(),
        }
    }
}

/// Run a periodic task with a tracing span, duration histogram, and
/// (on success) a row-count histogram.
///
/// The task closure must return `Result<u64, E>` where the `u64` is the
/// number of rows processed (return `0` if not meaningful).
pub async fn instrument_periodic_task<F, Fut, E>(
    instruments: &PeriodicTaskInstruments,
    task_name: &'static str,
    task_future: F,
) where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<u64, E>>,
    E: std::fmt::Display,
{
    let span = tracing::info_span!("periodic_task", task.name = task_name);
    let start = Instant::now();
    let result = task_future().instrument(span).await;
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let status = if result.is_ok() { "ok" } else { "err" };
    instruments.duration_ms.record(
        elapsed_ms,
        &[
            KeyValue::new("task.name", task_name),
            KeyValue::new("status", status),
        ],
    );
    match result {
        Ok(rows_affected) => instruments
            .rows_affected
            .record(rows_affected, &[KeyValue::new("task.name", task_name)]),
        Err(error) => {
            log::error!("periodic task '{task_name}' failed: {error}");
        }
    }
}

/// Flush and shut down OpenTelemetry providers gracefully.
pub fn shutdown_telemetry() {
    if let Some(tracer_provider) = TRACER_PROVIDER.get()
        && let Err(err) = tracer_provider.shutdown()
    {
        eprintln!("failed to shutdown tracer provider: {err}");
    }
    if let Some(logger_provider) = LOGGER_PROVIDER.get()
        && let Err(err) = logger_provider.shutdown()
    {
        eprintln!("failed to shutdown logger provider: {err}");
    }
    if let Some(meter_provider) = METER_PROVIDER.get()
        && let Err(err) = meter_provider.shutdown()
    {
        eprintln!("failed to shutdown meter provider: {err}");
    }
}
