//! Logging and tracing setup shared by the binaries.
//!
//! Logs always go to stdout. Spans are also exported over OTLP/HTTP when
//! `OTEL_EXPORTER_OTLP_ENDPOINT` (or `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`) is set, for example
//! `http://tempo.monitoring.svc.cluster.local:4318`. Without it, nothing is exported.
//!
//! Spans must never record credentials: the Telegram bot token, the Toloka password or
//! Transmission auth. Instrument with `skip_all` and add fields explicitly.

use opentelemetry::global;
use opentelemetry::propagation::Injector;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tracing::Level;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

const OTLP_ENDPOINT_VARS: [&str; 2] = [
    "OTEL_EXPORTER_OTLP_ENDPOINT",
    "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
];

/// Keeps the span exporter alive. Call [`Telemetry::shutdown`] (or drop it) before the process
/// exits so buffered spans are flushed; `std::process::exit` skips destructors.
#[must_use]
pub struct Telemetry {
    provider: Option<SdkTracerProvider>,
}

impl Telemetry {
    pub fn shutdown(&mut self) {
        if let Some(provider) = self.provider.take() {
            if let Err(error) = provider.shutdown() {
                eprintln!("Failed to flush traces: {error}");
            }
        }
    }
}

impl Drop for Telemetry {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn init(service_name: &'static str, log_level: Level) -> Telemetry {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // The exporter's own HTTP stack and the SDK internals must not feed back into traces,
    // and their debug chatter isn't useful in the app logs either.
    let quiet_internals = |default: Level| {
        Targets::new()
            .with_default(default)
            .with_target("opentelemetry", Level::WARN)
            .with_target("hyper", LevelFilter::OFF)
            .with_target("h2", LevelFilter::OFF)
            .with_target("reqwest", LevelFilter::OFF)
    };

    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(quiet_internals(log_level));

    let provider = OTLP_ENDPOINT_VARS
        .iter()
        .any(|name| std::env::var_os(name).is_some())
        .then(|| {
            let exporter = SpanExporter::builder()
                .with_http()
                .build()
                .expect("Failed to create OTLP span exporter");

            SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .with_resource(Resource::builder().with_service_name(service_name).build())
                .build()
        });

    let otel_layer = provider.as_ref().map(|provider| {
        tracing_opentelemetry::layer()
            .with_tracer(provider.tracer(service_name))
            .with_filter(quiet_internals(Level::INFO))
    });

    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(otel_layer);
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    Telemetry { provider }
}

/// Adds the W3C `traceparent` header for the current span, so the receiving service joins
/// the same trace.
pub fn trace_context_headers() -> HeaderMap {
    struct HeaderInjector<'a>(&'a mut HeaderMap);

    impl Injector for HeaderInjector<'_> {
        fn set(&mut self, key: &str, value: String) {
            if let (Ok(name), Ok(value)) = (HeaderName::try_from(key), HeaderValue::try_from(value))
            {
                self.0.insert(name, value);
            }
        }
    }

    let mut headers = HeaderMap::new();
    let context = tracing::Span::current().context();
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut HeaderInjector(&mut headers))
    });
    headers
}
