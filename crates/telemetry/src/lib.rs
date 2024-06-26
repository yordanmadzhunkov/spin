use std::io::IsTerminal;

use env::otel_enabled;
use env::otel_sdk_disabled;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter, Layer};

pub mod detector;
mod env;
mod propagation;
mod traces;

pub use propagation::extract_trace_context;
pub use propagation::inject_trace_context;

/// Initializes telemetry for Spin using the [tracing] library.
///
/// Under the hood this involves initializing a [tracing::Subscriber] with multiple [Layer]s. One
/// [Layer] emits [tracing] events to stderr, and another sends spans to an OTEL collector.
///
/// Configuration is pulled from the environment.
pub fn init(spin_version: String) -> anyhow::Result<ShutdownGuard> {
    // This layer will print all tracing library log messages to stderr.
    let fmt_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .with_filter(
            EnvFilter::from_default_env()
                .add_directive("wasmtime_wasi_http=warn".parse()?)
                .add_directive("watchexec=off".parse()?),
        );

    // We only want to build the otel layer if the user passed some endpoint configuration and it wasn't explicitly disabled.
    let build_otel_layer = !otel_sdk_disabled() && otel_enabled();
    let otel_layer = if build_otel_layer {
        // In this case we want to set the error handler to log errors to the tracing layer.
        opentelemetry::global::set_error_handler(otel_error_handler)?;

        Some(traces::otel_tracing_layer(spin_version)?)
    } else {
        None
    };

    // Build a registry subscriber with the layers we want to use.
    registry().with(otel_layer).with(fmt_layer).init();

    // Used to propagate trace information in the standard W3C TraceContext format. Even if the otel
    // layer is disabled we still want to propagate trace context.
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    Ok(ShutdownGuard)
}

fn otel_error_handler(err: opentelemetry::global::Error) {
    static FIRST_OTEL_ERROR: std::sync::Once = std::sync::Once::new();
    FIRST_OTEL_ERROR.call_once(|| {
        tracing::error!("There has been an error with the OpenTelemetry system, traces and metrics are likely failing to export");
        tracing::error!("Further OpenTelemetry errors will be logged at DEBUG level")
    });
    tracing::debug!(?err, "OpenTelemetry error");
}

/// An RAII implementation for connection to open telemetry services.
///
/// Shutdown of the open telemetry services will happen on `Drop`.
#[must_use]
pub struct ShutdownGuard;

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        // Give tracer provider a chance to flush any pending traces.
        opentelemetry::global::shutdown_tracer_provider();
    }
}
