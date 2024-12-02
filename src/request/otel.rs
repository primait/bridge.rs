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

#[cfg(feature = "tracing_opentelemetry_0_25")]
mod otel_0_25 {
    pub use opentelemetry_0_25_pkg::propagation::{Injector, TextMapPropagator};
    pub use opentelemetry_sdk_0_25_pkg::propagation::TraceContextPropagator;
    pub use tracing_opentelemetry_0_26_pkg::OpenTelemetrySpanExt;

    pub fn inject_context(injector: &mut dyn Injector) {
        TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
    }
}

#[cfg(feature = "tracing_opentelemetry_0_26")]
mod otel_0_26 {
    pub use opentelemetry_0_26_pkg::propagation::{Injector, TextMapPropagator};
    pub use opentelemetry_sdk_0_26_pkg::propagation::TraceContextPropagator;
    pub use tracing_opentelemetry_0_27_pkg::OpenTelemetrySpanExt;

    pub fn inject_context(injector: &mut dyn Injector) {
        TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
    }
}

#[cfg(feature = "tracing_opentelemetry_0_27")]
mod otel_0_26 {
    pub use opentelemetry_0_27_pkg::propagation::{Injector, TextMapPropagator};
    pub use opentelemetry_sdk_0_27_pkg::propagation::TraceContextPropagator;
    pub use tracing_opentelemetry_0_28_pkg::OpenTelemetrySpanExt;

    pub fn inject_context(injector: &mut dyn Injector) {
        TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
    }
}

#[cfg(feature = "tracing_opentelemetry_0_21")]
pub use otel_0_21::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_22")]
pub use otel_0_22::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_23")]
pub use otel_0_23::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_24")]
pub use otel_0_24::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_25")]
pub use otel_0_25::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_26")]
pub use otel_0_26::inject_context;

#[cfg(feature = "tracing_opentelemetry_0_27")]
pub use otel_0_26::inject_context;
