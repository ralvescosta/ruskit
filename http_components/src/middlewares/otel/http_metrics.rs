//! # Metrics Middleware

use super::attributes::metrics_attributes_from_request;
use actix_web::dev;
use futures_util::future::{self, FutureExt as _, LocalBoxFuture};
use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, Meter, Unit, UpDownCounter};
use std::{sync::Arc, time::SystemTime};

// Follows the experimental semantic conventions for HTTP metrics:
// https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/semantic_conventions/http-metrics.md
use opentelemetry_semantic_conventions::trace::HTTP_RESPONSE_STATUS_CODE;
const HTTP_SERVER_ACTIVE_REQUESTS: &str = "http.server.active_requests";
const HTTP_SERVER_DURATION: &str = "http.server.duration";
const HTTP_SERVER_REQUESTS: &str = "http.server.requests";

#[derive(Clone, Debug)]
struct Metrics {
    http_server_active_requests: UpDownCounter<i64>,
    http_server_duration: Histogram<f64>,
    http_requests: Counter<u64>,
}

impl Metrics {
    fn new(meter: Meter) -> Self {
        let http_server_active_requests = meter
            .i64_up_down_counter(HTTP_SERVER_ACTIVE_REQUESTS)
            .with_description("HTTP concurrent in-flight requests per route")
            .init();

        let http_server_duration = meter
            .f64_histogram(HTTP_SERVER_DURATION)
            .with_description("HTTP inbound request duration per route")
            .with_unit(Unit::new("ms"))
            .init();

        let http_requests = meter
            .u64_counter(HTTP_SERVER_REQUESTS)
            .with_description("HTTP Requests")
            .init();

        Metrics {
            http_server_active_requests,
            http_server_duration,
            http_requests,
        }
    }
}

/// Request metrics tracking
///
/// # Examples
///
/// ```no_run
/// use actix_web::{dev, http, web, App, HttpRequest, HttpServer};
/// use http_components::{
///     middlewares::otel::{
///         HTTPOtelMetrics,
///     },
///     handlers::PrometheusMetricsHandler
/// };
/// use opentelemetry::{
///     global,
///     sdk::{
///         export::metrics::aggregation,
///         metrics::{controllers, processors, selectors},
///         propagation::TraceContextPropagator,
///     },
/// };
///
/// # #[cfg(feature = "metrics-prometheus")]
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     // Request metrics middleware
///     let meter = global::meter("actix_web");
///     let request_metrics = RequestMetricsBuilder::new().build(meter);
///
///     // Prometheus request metrics handler
///     let controller = controllers::basic(
///         processors::factory(
///             selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
///             aggregation::cumulative_temporality_selector(),
///         )
///         .with_memory(true),
///     )
///     .build();
///     let exporter = opentelemetry_prometheus::exporter(controller).init();
///     let metrics_handler = PrometheusMetricsHandler::new(exporter);
///
///     // Run actix server, metrics are now available at http://localhost:8080/metrics
///     HttpServer::new(move || {
///         App::new()
///             .wrap(HTTPOtelMetrics::new())
///             .wrap(request_metrics.clone())
///             .route("/metrics", web::get().to(metrics_handler.clone()))
///     })
///     .bind("localhost:8080")?
///     .run()
///     .await
/// }
/// ```
#[derive(Clone, Debug)]
pub struct HTTPOtelMetrics {
    metrics: Arc<Metrics>,
}

impl HTTPOtelMetrics {
    pub fn new() -> HTTPOtelMetrics {
        HTTPOtelMetrics {
            metrics: Arc::new(Metrics::new(global::meter("http-meter-middleware"))),
        }
    }
}

impl<S, B> dev::Transform<S, dev::ServiceRequest> for HTTPOtelMetrics
where
    S: dev::Service<
        dev::ServiceRequest,
        Response = dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = HTTPOtelMetricsMiddleware<S>;
    type InitError = ();
    type Future = future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let service = HTTPOtelMetricsMiddleware {
            service,
            metrics: self.metrics.clone(),
        };

        future::ok(service)
    }
}

/// Request metrics middleware
#[allow(missing_debug_implementations)]
pub struct HTTPOtelMetricsMiddleware<S> {
    service: S,
    metrics: Arc<Metrics>,
}

impl<S, B> dev::Service<dev::ServiceRequest> for HTTPOtelMetricsMiddleware<S>
where
    S: dev::Service<
        dev::ServiceRequest,
        Response = dev::ServiceResponse<B>,
        Error = actix_web::Error,
    >,
    S::Future: 'static,
    B: 'static,
{
    type Response = dev::ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: dev::ServiceRequest) -> Self::Future {
        let timer = SystemTime::now();

        let http_target = req.match_pattern().unwrap_or_else(|| "default".to_string());

        let mut attributes = metrics_attributes_from_request(&req, &http_target);

        self.metrics.http_server_active_requests.add(1, &attributes);

        let metrics = self.metrics.clone();
        Box::pin(self.service.call(req).map(move |res| {
            metrics.http_server_active_requests.add(-1, &attributes);

            // Ignore actix errors for metrics
            match res {
                Ok(success) => {
                    attributes.push(
                        HTTP_RESPONSE_STATUS_CODE.string(success.status().as_str().to_owned()),
                    );

                    metrics.http_server_duration.record(
                        timer
                            .elapsed()
                            .map(|t| t.as_secs_f64() * 1000.0)
                            .unwrap_or_default(),
                        &attributes,
                    );

                    metrics.http_requests.add(1, &attributes);

                    Ok(success)
                }
                Err(err) => {
                    attributes.push(
                        HTTP_RESPONSE_STATUS_CODE
                            .string(err.as_response_error().status_code().as_str().to_owned()),
                    );

                    metrics.http_requests.add(1, &attributes);

                    Err(err)
                }
            }
        }))
    }
}
