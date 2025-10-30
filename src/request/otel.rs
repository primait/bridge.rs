#[cfg(feature = "tracing_opentelemetry_0_30")]
mod otel_crates {
    pub use opentelemetry_0_30_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_30_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_31_pkg as tracing_opentelemetry;
}

#[cfg(feature = "tracing_opentelemetry_0_31")]
mod otel_crates {
    pub use opentelemetry_0_31_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_31_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_32_pkg as tracing_opentelemetry;
}

use otel_crates::*;

pub use opentelemetry::propagation::Injector;

use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub fn inject_context(injector: &mut dyn Injector) {
    TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
}
