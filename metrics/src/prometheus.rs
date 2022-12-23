use actix_web::{web::Data, HttpResponse, Responder};
use env::Config;
use opentelemetry::sdk::{
    export::metrics::aggregation,
    metrics::{controllers, processors, selectors},
};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::error::Error;
use std::sync::Arc;
use tracing::debug;

pub fn setup(cfg: &Config) -> Result<Option<PrometheusExporter>, Box<dyn Error>> {
    if !cfg.otlp.enable_metrics {
        debug!("metrics::setup skipping metrics export setup");
        return Ok(None);
    }

    debug!("metrics::setup configure prometheus...");

    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();

    let exporter = opentelemetry_prometheus::exporter(controller).init();

    debug!("metrics::setup installed");

    Ok(Some(exporter))
}

pub async fn metrics_handler(exporter: Data<Arc<PrometheusExporter>>) -> impl Responder {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();

    let metric_families = exporter.registry().gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}
