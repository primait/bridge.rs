use opentelemetry::{
    propagation::{Injector, TextMapPropagator},
    sdk::propagation::TraceContextPropagator,
};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub fn inject_context(injector: &mut dyn Injector) {
    TraceContextPropagator::new().inject_context(&tracing::Span::current().context(), injector);
}
