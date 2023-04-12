//! # Metrics Middleware

use super::attributes::metrics_attributes_from_request;
use actix_web::dev;
use futures_util::future::{self, FutureExt as _, LocalBoxFuture};
use opentelemetry::metrics::{Histogram, Meter, Unit, UpDownCounter};
use opentelemetry::{global, Context};
use std::{sync::Arc, time::SystemTime};

// Follows the experimental semantic conventions for HTTP metrics:
// https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/metrics/semantic_conventions/http-metrics.md
use opentelemetry_semantic_conventions::trace::HTTP_STATUS_CODE;
const HTTP_SERVER_ACTIVE_REQUESTS: &str = "http.server.active_requests";
const HTTP_SERVER_DURATION: &str = "http.server.duration";

#[derive(Clone, Debug)]
struct Metrics {
    http_server_active_requests: UpDownCounter<i64>,
    http_server_duration: Histogram<f64>,
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

        Metrics {
            http_server_active_requests,
            http_server_duration,
        }
    }
}

/// Request metrics tracking
///
/// # Examples
///
/// ```no_run
/// use actix_web::{dev, http, web, App, HttpRequest, HttpServer};
/// use actix_web_opentelemetry::{
///     PrometheusMetricsHandler,
///     RequestMetricsBuilder,
///     RequestTracing,
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
///             .wrap(RequestTracing::new())
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
        let cx = Context::current();

        self.metrics
            .http_server_active_requests
            .add(&cx, 1, &attributes);

        let request_metrics = self.metrics.clone();
        Box::pin(self.service.call(req).map(move |res| {
            request_metrics
                .http_server_active_requests
                .add(&cx, -1, &attributes);

            // Ignore actix errors for metrics
            if let Ok(res) = res {
                attributes.push(HTTP_STATUS_CODE.string(res.status().as_str().to_owned()));

                request_metrics.http_server_duration.record(
                    &cx,
                    timer
                        .elapsed()
                        .map(|t| t.as_secs_f64() * 1000.0)
                        .unwrap_or_default(),
                    &attributes,
                );

                Ok(res)
            } else {
                res
            }
        }))
    }
}

#[cfg(feature = "metrics-prometheus")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics-prometheus")))]
pub(crate) mod prometheus {
    use actix_web::{dev, http::StatusCode};
    use futures_util::future::{self, LocalBoxFuture};
    use opentelemetry::{global, metrics::MetricsError};
    use opentelemetry_prometheus::PrometheusExporter;
    use prometheus::{Encoder, TextEncoder};

    /// Prometheus request metrics service
    #[derive(Clone, Debug)]
    pub struct PrometheusMetricsHandler {
        prometheus_exporter: PrometheusExporter,
    }

    impl PrometheusMetricsHandler {
        /// Build a route to serve Prometheus metrics
        pub fn new(exporter: PrometheusExporter) -> Self {
            Self {
                prometheus_exporter: exporter,
            }
        }
    }

    impl PrometheusMetricsHandler {
        fn metrics(&self) -> String {
            let encoder = TextEncoder::new();
            let metric_families = self.prometheus_exporter.registry().gather();
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
}
