use std::str::FromStr;

use tonic::metadata::{MetadataKey, MetadataMap, MetadataValue};
use tonic::service::interceptor::InterceptedService;
use tonic::service::Interceptor;
use tonic::{Request, Status};

use super::otel::{inject_context, Injector};

/// A gRPC interceptor that injects the current tracing context into the request metadata.
///
/// Use this interceptor to make sure that DataDog traces are connected between the services when calling the gRPC
/// service on another server.
///
/// # Example
///
/// ```
/// # struct QuotePreviewServiceClient<S> { _d: std::marker::PhantomData<S> }
/// # impl QuotePreviewServiceClient<GrpcOtelInterceptedService<Channel>> {
/// #     pub fn with_interceptor(endpoint: Channel, interceptor: GrpcOtelInterceptor) -> Self { Self { _d: std::marker::PhantomData } }
/// # }
/// use tonic::transport::{Endpoint, Channel};
///
/// use prima_bridge::{GrpcOtelInterceptor, GrpcOtelInterceptedService};
///
/// async fn make_grpc_service() -> Result<QuotePreviewServiceClient<GrpcOtelInterceptedService<Channel>>, Box<dyn std::error::Error>> {
///     let url = "http://...";
///     let channel = Endpoint::new(url)?.connect().await?;
///     // QuotePreviewServiceClient is a tonic-generated gRPC client from [https://github.com/primait/es-engine-schema]
///     Ok(QuotePreviewServiceClient::with_interceptor(channel, GrpcOtelInterceptor))
/// }
/// ```
#[derive(Clone)]
pub struct GrpcOtelInterceptor;

/// Convenience type alias for a long type that's returned from the `with_interceptor()` function of the tonic-generated
/// gRPC client when used with [GrpcOtelInterceptor].
pub type GrpcOtelInterceptedService<Svc> = InterceptedService<Svc, GrpcOtelInterceptor>;

impl Interceptor for GrpcOtelInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        inject_context(&mut MetadataMapInjector(request.metadata_mut()));
        Ok(request)
    }
}

pub struct MetadataMapInjector<'h>(&'h mut MetadataMap);

impl Injector for MetadataMapInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        let key_value = MetadataKey::from_str(key)
            .ok()
            .and_then(|key| Some((key, MetadataValue::from_str(&value).ok()?)));
        if let Some((key, value)) = key_value {
            self.0.insert(key, value);
        }
    }
}
