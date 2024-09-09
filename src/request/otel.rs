#[cfg(feature = "tracing_opentelemetry_0_20")]
mod otel_0_20 {
    pub use opentelemetry_0_20_pkg::{
        propagation::{Injector, TextMapPropagator},
        sdk::propagation::TraceContextPropagator,
    };
    pub use tracing_opentelemetry_0_21_pkg::OpenTelemetrySpanExt;

    pub fn inject_context(injector: &mut dyn Injector) {
        TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
    }
}

#[cfg(feature = "tracing_opentelemetry_0_21")]
mod otel_0_21 {
    pub use opentelemetry_0_21_pkg::propagation::{Injector, TextMapPropagator};
    pub use opentelemetry_sdk_0_21_pkg::propagation::TraceContextPropagator;
    pub use tracing_opentelemetry_0_22_pkg::OpenTelemetrySpanExt;

    pub fn inject_context(injector: &mut dyn Injector) {
        TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
    }
}

#[cfg(feature = "tracing_opentelemetry_0_22")]
mod otel_0_22 {
    pub use opentelemetry_0_22_pkg::propagation::{Injector, TextMapPropagator};
    pub use opentelemetry_sdk_0_22_pkg::propagation::TraceContextPropagator;
    pub use tracing_opentelemetry_0_23_pkg::OpenTelemetrySpanExt;

    pub fn inject_context(injector: &mut dyn Injector) {
        TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
    }
}

#[cfg(feature = "tracing_opentelemetry_0_23")]
mod otel_0_23 {
    pub use opentelemetry_0_23_pkg::propagation::{Injector, TextMapPropagator};
    pub use opentelemetry_sdk_0_23_pkg::propagation::TraceContextPropagator;
    pub use tracing_opentelemetry_0_24_pkg::OpenTelemetrySpanExt;

    pub fn inject_context(injector: &mut dyn Injector) {
        TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
    }
}

#[cfg(feature = "tracing_opentelemetry_0_24")]
mod otel_0_24 {
    pub use opentelemetry_0_24_pkg::propagation::{Injector, TextMapPropagator};
    pub use opentelemetry_sdk_0_24_pkg::propagation::TraceContextPropagator;
    pub use tracing_opentelemetry_0_25_pkg::OpenTelemetrySpanExt;

    pub fn inject_context(injector: &mut dyn Injector) {
        TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
    }
}

#[cfg(feature = "tracing_opentelemetry_0_20")]
pub use otel_0_20::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_21")]
pub use otel_0_21::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_22")]
pub use otel_0_22::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_23")]
pub use otel_0_23::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_24")]
pub use otel_0_24::inject_context;
