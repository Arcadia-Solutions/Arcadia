use opentelemetry::KeyValue;
use opentelemetry::metrics::Counter;
use std::sync::OnceLock;

static INSTRUMENTS: OnceLock<Instruments> = OnceLock::new();

#[derive(Debug)]
pub struct Instruments {
    pub errors: Counter<u64>,
}

pub fn register(service_name: &str) {
    let scope = opentelemetry::InstrumentationScope::builder(service_name.to_string()).build();
    let meter = opentelemetry::global::meter_with_scope(scope);

    let instruments = Instruments {
        errors: meter
            .u64_counter("http.errors")
            .with_description("Total number of HTTP error responses")
            .build(),
    };

    let _ = INSTRUMENTS.set(instruments);
}

pub fn record_error(error_variant: &str, status_code: u16) {
    if let Some(instruments) = INSTRUMENTS.get() {
        instruments.errors.add(
            1,
            &[
                KeyValue::new("error", error_variant.to_string()),
                KeyValue::new("status_code", i64::from(status_code)),
            ],
        );
    }
}
