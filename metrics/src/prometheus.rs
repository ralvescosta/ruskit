use configs::{Configs, DynamicConfigs};
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{metrics::MeterProvider, Resource};
use prometheus::Registry;
use std::{error::Error, sync::Arc};
use tracing::debug;

pub fn setup<T>(cfg: &Configs<T>) -> Result<Arc<Registry>, Box<dyn Error>>
where
    T: DynamicConfigs,
{
    let registry = Registry::new();

    if !cfg.metric.enable {
        debug!("metrics::setup skipping metrics export setup");
        return Ok(Arc::new(registry));
    }

    debug!("metrics::setup configure prometheus metrics...");

    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()?;

    let provider = MeterProvider::builder()
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", cfg.app.name.clone()),
            KeyValue::new("service.type", cfg.metric.service_type.clone()),
            KeyValue::new("environment", format!("{}", cfg.app.env)),
            KeyValue::new("library.language", "rust"),
        ]))
        .with_reader(exporter)
        .build();

    global::set_meter_provider(provider);

    debug!("metrics::setup prometheus installed");

    Ok(Arc::new(registry))
}
