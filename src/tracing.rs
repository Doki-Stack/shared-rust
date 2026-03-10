use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

use crate::error::{Error, Result};

/// Initialize tracing with JSON formatter and env filter.
///
/// Uses `RUST_LOG` for filter (default: `info`).
pub fn init_tracing(service_name: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_target(true)
        .with_current_span(false)
        .with_span_list(false);

    let registry = tracing_subscriber::registry().with(filter).with(fmt_layer);

    registry
        .try_init()
        .map_err(|e| Error::internal(format!("Failed to init tracing: {}", e)))?;

    tracing::info!(service = %service_name, "Tracing initialized");
    Ok(())
}
