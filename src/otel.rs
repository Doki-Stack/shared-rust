use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::{Error, Result};

/// Guard that shuts down OTel on drop.
pub struct OtelGuard;

impl Drop for OtelGuard {
    fn drop(&mut self) {
        global::shutdown_tracer_provider();
    }
}

/// Initialize OpenTelemetry with OTLP gRPC exporter.
///
/// Env: `OTEL_EXPORTER_OTLP_ENDPOINT` (default: `http://localhost:4317`)
pub fn init_otel(service_name: &str) -> Result<OtelGuard> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&endpoint),
        )
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
            Resource::new(vec![KeyValue::new(
                "service.name",
                service_name.to_string(),
            )]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(|e| Error::internal(format!("Failed to init OTel tracer: {}", e)))?;

    let tracer = provider.tracer("doki-shared");
    global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());

    let otel_layer = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(otel_layer)
        .try_init()
        .map_err(|e| Error::internal(format!("Failed to init OTel subscriber: {}", e)))?;

    tracing::info!(endpoint = %endpoint, "OpenTelemetry initialized");

    Ok(OtelGuard)
}

/// Get trace_id from current span context.
pub fn trace_id_from_context() -> Option<String> {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let context = tracing::Span::current().context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();

    if span_context.is_valid() {
        Some(span_context.trace_id().to_string())
    } else {
        None
    }
}

/// Get span_id from current span context.
pub fn span_id_from_context() -> Option<String> {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let context = tracing::Span::current().context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();

    if span_context.is_valid() {
        Some(span_context.span_id().to_string())
    } else {
        None
    }
}
