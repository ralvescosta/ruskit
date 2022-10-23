use crate::get_sampler;
use env::Config;
use opentelemetry::{
    global, runtime,
    sdk::{
        propagation::{BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator},
        trace::{self, RandomIdGenerator},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use std::{error::Error, time::Duration, vec};
use tonic::metadata::*;
use tracing::{debug, error};

pub fn setup(cfg: &Config) -> Result<(), Box<dyn Error>> {
    if !cfg.enable_traces {
        debug!("traces::setup skipping trace export setup");
        return Ok(());
    }

    debug!("traces::setup starting traces setup...");

    let mut map = MetadataMap::with_capacity(3);

    match cfg.otlp_key.parse() {
        Ok(p) => {
            map.insert("api-key", p);
            Ok(())
        }
        Err(e) => {
            error!(error = e.to_string(), "err mapping otlp key from config");
            Err(e)
        }
    }?;

    debug!("traces::setup creating the tracer...");

    let config = trace::config()
        .with_sampler(get_sampler(cfg))
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", cfg.app_name.clone()),
            KeyValue::new("service.type", cfg.otlp_service_type.clone()),
            KeyValue::new("environment", format!("{}", cfg.env)),
            KeyValue::new("library.language", "rust"),
        ]));

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&cfg.otlp_host)
        .with_protocol(Protocol::Grpc)
        .with_timeout(Duration::from_secs(cfg.otlp_export_timeout))
        .with_metadata(map);

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(config)
        .with_exporter(exporter)
        .install_batch(runtime::Tokio)
        .map_err(|e| {
            error!(error = e.to_string(), "err installing otlp tracing");
            e
        })?;

    global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
        Box::new(TraceContextPropagator::new()),
        Box::new(BaggagePropagator::new()),
    ]));

    debug!("traces::setup tracer installed");

    Ok(())
}
