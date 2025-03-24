#[cfg(feature = "tracing_opentelemetry_0_21")]
mod otel_crates {
    pub use opentelemetry_0_21_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_21_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_22_pkg as tracing_opentelemetry;
}

#[cfg(feature = "tracing_opentelemetry_0_22")]
mod otel_crates {
    pub use opentelemetry_0_22_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_22_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_23_pkg as tracing_opentelemetry;
}

#[cfg(feature = "tracing_opentelemetry_0_23")]
mod otel_crates {
    pub use opentelemetry_0_23_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_23_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_24_pkg as tracing_opentelemetry;
}

#[cfg(feature = "tracing_opentelemetry_0_24")]
mod otel_crates {
    pub use opentelemetry_0_24_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_24_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_25_pkg as tracing_opentelemetry;
}

#[cfg(feature = "tracing_opentelemetry_0_25")]
mod otel_crates {
    pub use opentelemetry_0_25_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_25_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_26_pkg as tracing_opentelemetry;
}

#[cfg(feature = "tracing_opentelemetry_0_26")]
mod otel_crates {
    pub use opentelemetry_0_26_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_26_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_27_pkg as tracing_opentelemetry;
}

#[cfg(feature = "tracing_opentelemetry_0_27")]
mod otel_crates {
    pub use opentelemetry_0_27_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_27_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_28_pkg as tracing_opentelemetry;
}

#[cfg(feature = "tracing_opentelemetry_0_28")]
mod otel_crates {
    pub use opentelemetry_0_28_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_28_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_29_pkg as tracing_opentelemetry;
}

#[cfg(feature = "tracing_opentelemetry_0_29")]
mod otel_crates {
    pub use opentelemetry_0_29_pkg as opentelemetry;
    pub use opentelemetry_sdk_0_29_pkg as opentelemetry_sdk;
    pub use tracing_opentelemetry_0_30_pkg as tracing_opentelemetry;
}

use otel_crates::*;

pub use opentelemetry::propagation::Injector;

use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub fn inject_context(injector: &mut dyn Injector) {
    TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
}
