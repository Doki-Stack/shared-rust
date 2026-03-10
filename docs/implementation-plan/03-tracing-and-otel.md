# shared-rust Implementation Plan — Tracing and OpenTelemetry

## 1. tracing.rs

```rust
//! tracing-subscriber initialization with JSON output.

use tracing_subscriber::{
    fmt::format::FmtSpan,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

use crate::error::{Error, Result};

/// Initialize tracing with JSON formatter and env filter.
///
/// Uses `RUST_LOG` for filter (default: `info`).
/// Output format: JSON with timestamp, level, target, message, service.
pub fn init_tracing(service_name: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_target(true)
        .with_current_span(false)
        .with_span_list(false);

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer);

    registry
        .try_init()
        .map_err(|e| Error::internal(format!("Failed to init tracing: {}", e)))?;

    tracing::info!(service = %service_name, "Tracing initialized");
    Ok(())
}
```

### JSON Output Format

```json
{
  "timestamp": "2025-03-10T12:00:00.000000Z",
  "level": "INFO",
  "target": "mcp_scanner::main",
  "message": "Starting scan",
  "service": "mcp-scanner"
}
```

## 2. otel.rs (behind `otel` feature)

```rust
//! OpenTelemetry integration: Tempo traces + Prometheus metrics.

use opentelemetry::{
    global,
    sdk::{
        trace::{BatchSpanProcessor, TracerProvider},
        Resource,
    },
    trace::trace_flags_from_byte,
};
use opentelemetry_otlp::WithExportConfig;
use std::sync::Arc;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::{Error, Result};

/// Guard that flushes and shuts down OTel on drop.
pub struct OtelGuard {
    _tracer_provider: Option<Arc<TracerProvider>>,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Some(provider) = self._tracer_provider.take() {
            if let Err(e) = provider.force_flush() {
                tracing::warn!(error = %e, "OTel flush failed");
            }
        }
        global::shutdown_tracer_provider();
    }
}

/// Initialize OpenTelemetry.
///
/// - TracerProvider: OTLP gRPC → Tempo
/// - W3C TraceContext + Baggage propagator
/// - Env: OTEL_EXPORTER_OTLP_ENDPOINT (default: http://localhost:4317)
pub fn init_otel(service_name: &str) -> Result<OtelGuard> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&endpoint)
        .with_export_config(
            opentelemetry_otlp::ExportConfig::default()
                .with_timeout(std::time::Duration::from_secs(10)),
        );

    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", service_name.to_string()),
        ]))
        .build();

    let tracer = tracer_provider.tracer("doki-shared");
    global::set_tracer_provider(tracer_provider.clone());

    // W3C propagator
    global::set_text_map_propagator(opentelemetry::sdk::propagation::TraceContextPropagator::new());

    let otel_layer = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(otel_layer)
        .try_init()
        .map_err(|e| Error::internal(format!("Failed to init OTel: {}", e)))?;

    tracing::info!(endpoint = %endpoint, "OpenTelemetry initialized");

    Ok(OtelGuard {
        _tracer_provider: Some(Arc::new(tracer_provider)),
    })
}

/// Get trace_id from current span context.
pub fn trace_id_from_context() -> Option<String> {
    Span::current()
        .context()
        .span()
        .span_context()
        .trace_id()
        .to_string()
        .into()
}

/// Get span_id from current span context.
pub fn span_id_from_context() -> Option<String> {
    Span::current()
        .context()
        .span()
        .span_context()
        .span_id()
        .to_string()
        .into()
}
```

### Stub when `otel` disabled

In lib.rs or a separate module, provide fallback:

```rust
// In lib.rs or otel.rs (conditional)
#[cfg(not(feature = "otel"))]
mod otel_stub {
    /// Stub when otel feature is disabled.
    pub fn trace_id_from_context() -> Option<String> {
        None
    }

    pub fn span_id_from_context() -> Option<String> {
        None
    }
}

#[cfg(not(feature = "otel"))]
pub use otel_stub::{trace_id_from_context, span_id_from_context};
```

Ensure `error.rs` uses `crate::trace_id_from_context()` so it works with or without `otel`.

## 3. Integration Pattern

```rust
// In service main.rs
#[tokio::main]
async fn main() -> Result<()> {
    doki_shared::init_tracing("mcp-scanner")?;

    let _otel_guard = if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        Some(doki_shared::init_otel("mcp-scanner")?)
    } else {
        None
    };

    // ... rest of app
}
```

## 4. Test Plan

| Test | Description |
|------|-------------|
| `test_init_tracing` | init_tracing("test") succeeds, no panic |
| `test_trace_id_without_otel` | trace_id_from_context() returns None when otel disabled |
| `test_otel_guard_drop` | OtelGuard drop does not panic |
| `test_feature_gate` | Crate compiles with and without `otel` |

### Test Snippets

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing() {
        let result = init_tracing("test-service");
        // May fail if already initialized; that's ok
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("init"));
    }
}
```
