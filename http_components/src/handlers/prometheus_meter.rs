use actix_http::body::BoxBody;
use actix_web::dev;
use futures_util::future::{self, LocalBoxFuture};
use opentelemetry::{global, metrics::MetricsError};
use prometheus::{Encoder, Registry, TextEncoder};
use std::sync::Arc;

/// Prometheus request metrics service
#[derive(Clone, Debug)]
pub struct PrometheusMetricsHandler {
    registry: Arc<Registry>,
}

impl PrometheusMetricsHandler {
    /// Build a route to serve Prometheus metrics
    pub fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }
}

impl PrometheusMetricsHandler {
    fn metrics(&self) -> (String, String) {
        let encoder = TextEncoder::new();

        let metric_families = self.registry.gather();
        let mut buf = Vec::new();
        if let Err(err) = encoder.encode(&metric_families[..], &mut buf) {
            global::handle_error(MetricsError::Other(err.to_string()));
        }

        (
            String::from_utf8(buf).unwrap_or_default(),
            encoder.format_type().to_owned(),
        )
    }
}

impl dev::Handler<actix_web::HttpRequest> for PrometheusMetricsHandler {
    type Output = Result<actix_web::HttpResponse<BoxBody>, actix_web::error::Error>;
    type Future = LocalBoxFuture<'static, Self::Output>;

    fn call(&self, _req: actix_web::HttpRequest) -> Self::Future {
        let (metrics, content_type) = self.metrics();

        Box::pin(future::ok(
            actix_web::HttpResponse::Ok()
                .content_type(content_type)
                .body::<String>(metrics),
        ))
    }
}
