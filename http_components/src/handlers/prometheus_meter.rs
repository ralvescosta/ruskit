use actix_web::{dev, http::StatusCode};
use futures_util::future::{self, LocalBoxFuture};
use opentelemetry::{global, metrics::MetricsError};
use prometheus::{Encoder, Registry, TextEncoder};

/// Prometheus request metrics service
#[derive(Clone, Debug)]
pub struct PrometheusMetricsHandler {
    registry: Registry,
}

impl PrometheusMetricsHandler {
    /// Build a route to serve Prometheus metrics
    pub fn new(registry: Registry) -> Self {
        Self { registry }
    }
}

impl PrometheusMetricsHandler {
    fn metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buf = Vec::new();
        if let Err(err) = encoder.encode(&metric_families[..], &mut buf) {
            global::handle_error(MetricsError::Other(err.to_string()));
        }

        String::from_utf8(buf).unwrap_or_default()
    }
}

impl dev::Handler<actix_web::HttpRequest> for PrometheusMetricsHandler {
    type Output = Result<actix_web::HttpResponse<String>, actix_web::error::Error>;
    type Future = LocalBoxFuture<'static, Self::Output>;

    fn call(&self, _req: actix_web::HttpRequest) -> Self::Future {
        Box::pin(future::ok(actix_web::HttpResponse::with_body(
            StatusCode::OK,
            self.metrics(),
        )))
    }
}
