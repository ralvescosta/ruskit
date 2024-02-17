use crate::{errors::TracesError, get_sampler};
use configs::{Configs, DynamicConfigs, TraceExporterKind};
use opentelemetry::KeyValue;
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator},
    Resource,
};
use std::vec;
use tracing::{debug, error};

#[cfg(any(feature = "otlp", feature = "stdout"))]
use crate::exporters;

pub fn init<T>(cfg: &Configs<T>) -> Result<(), TracesError>
where
    T: DynamicConfigs,
{
    if !cfg.trace.enable {
        debug!("traces::init skipping trace export setup");
        return Ok(());
    }

    debug!("traces::init creating the tracer...");

    let _trace_configs = trace::config()
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

    match cfg.trace.exporter {
        TraceExporterKind::Stdout => {
            #[cfg(feature = "stdout")]
            {
                exporters::stdout::install(_trace_configs)
            }

            #[cfg(not(feature = "stdout"))]
            {
                error!("stdout traces required to configure features = [stdout]");
                Err(TracesError::InvalidFeaturesError)
            }
        }
        TraceExporterKind::OtlpGrpc => {
            #[cfg(feature = "otlp")]
            {
                exporters::otlp_grpc::install(cfg, _trace_configs)
            }

            #[cfg(not(feature = "otlp"))]
            {
                error!("otlp traces required to configure features = [otlp]");
                Err(TracesError::InvalidFeaturesError)
            }
        }
    }
}
