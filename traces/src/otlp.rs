use crate::get_sampler;
use configs::{Configs, DynamicConfigs};
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

pub fn setup<T>(cfg: &Configs<T>) -> Result<(), Box<dyn Error>>
where
    T: DynamicConfigs,
{
    if !cfg.trace.enable {
        debug!("traces::setup skipping trace export setup");
        return Ok(());
    }

    debug!("traces::setup starting traces setup...");

    let mut map = MetadataMap::with_capacity(3);

    match cfg.trace.key.parse() {
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
            KeyValue::new("service.name", cfg.app.name.clone()),
            KeyValue::new("service.type", cfg.trace.service_type.clone()),
            KeyValue::new("environment", format!("{}", cfg.app.env)),
            KeyValue::new("library.language", "rust"),
        ]));

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&cfg.trace.host)
        .with_protocol(Protocol::Grpc)
        .with_timeout(Duration::from_secs(cfg.trace.export_timeout))
        .with_metadata(map);

    match opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(config)
        .with_exporter(exporter)
        .install_batch(runtime::Tokio)
    {
        Err(err) => {
            error!(error = err.to_string(), "err installing otlp tracing");
            Err(err.into())
        }
        _ => {
            global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
                Box::new(TraceContextPropagator::new()),
                Box::new(BaggagePropagator::new()),
            ]));

            debug!("traces::setup tracer installed");

            Ok(())
        }
    }
}
